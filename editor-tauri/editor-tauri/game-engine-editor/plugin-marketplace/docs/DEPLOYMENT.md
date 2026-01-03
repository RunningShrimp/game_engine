# Plugin Marketplace Deployment Guide

Complete guide for deploying the plugin marketplace system to production.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Database Setup](#database-setup)
3. [Backend Deployment](#backend-deployment)
4. [Frontend Deployment](#frontend-deployment)
5. [Storage Configuration](#storage-configuration)
6. [DNS and SSL](#dns-and-ssl)
7. [Monitoring and Logging](#monitoring-and-logging)
8. [Backup and Recovery](#backup-and-recovery)

## Prerequisites

### Required Services

- PostgreSQL 14+ database
- AWS S3 or compatible object storage
- Domain name
- SSL certificate

### Required Tools

- Docker and Docker Compose
- kubectl (if using Kubernetes)
- AWS CLI (if using AWS)

## Database Setup

### Option 1: Managed PostgreSQL (Recommended)

#### AWS RDS

```bash
# Create RDS instance
aws rds create-db-instance \
  --db-instance-identifier plugin-marketplace-db \
  --db-instance-class db.t3.micro \
  --engine postgres \
  --engine-version 14.7 \
  --allocated-storage 20 \
  --master-username admin \
  --master-user-password your-password \
  --vpc-security-group-ids sg-xxxxx \
  --db-subnet-group-name default-vpc-xxxxx

# Get connection string
aws rds describe-db-instances \
  --db-instance-identifier plugin-marketplace-db \
  --query "DBInstances[0].Endpoint.Address"
```

#### DigitalOcean Managed Database

```bash
# Create database cluster
doctl databases create plugin-marketplace-db \
  --engine pg \
  --version 14 \
  --num-nodes 1 \
  --size db-s-1vcpu-1gb \
  --region nyc1

# Get connection string
doctl databases get connection-string plugin-marketplace-db
```

### Option 2: Self-Hosted PostgreSQL

```bash
# Using Docker
docker run -d \
  --name plugin-marketplace-db \
  -e POSTGRES_USER=admin \
  -e POSTGRES_PASSWORD=secure-password \
  -e POSTGRES_DB=plugin_marketplace \
  -v pgdata:/var/lib/postgresql/data \
  -p 5432:5432 \
  postgres:14

# Run migrations
docker exec -it plugin-marketplace-db psql -U admin -d plugin_marketplace -f /migrations/001_initial.up.sql
```

### Database Configuration

Create `.env` file:

```env
DATABASE_URL=postgresql://admin:password@localhost:5432/plugin_marketplace
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
```

## Backend Deployment

### Option 1: Docker Deployment

#### 1. Build Docker Image

```bash
cd backend

# Build image
docker build -t plugin-marketplace-backend:v1.0.0 .

# Tag for registry
docker tag plugin-marketplace-backend:v1.0.0 registry.example.com/plugin-marketplace-backend:v1.0.0

# Push to registry
docker push registry.example.com/plugin-marketplace-backend:v1.0.0
```

#### 2. Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  backend:
    image: plugin-marketplace-backend:v1.0.0
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://admin:password@db:5432/plugin_marketplace
      - JWT_SECRET=${JWT_SECRET}
      - S3_BUCKET=${S3_BUCKET}
      - AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
      - AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}
      - AWS_REGION=${AWS_REGION}
      - RUST_LOG=info
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:14
    environment:
      - POSTGRES_USER=admin
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=plugin_marketplace
    volumes:
      - pgdata:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  pgdata:
```

Deploy:

```bash
docker-compose up -d
```

### Option 2: Kubernetes Deployment

#### 1. Create Namespace

```bash
kubectl create namespace plugin-marketplace
```

#### 2. Create Secrets

```bash
# Database secret
kubectl create secret generic db-secret \
  --from-literal=url='postgresql://admin:password@db:5432/plugin_marketplace' \
  --namespace=plugin-marketplace

# JWT secret
kubectl create secret generic jwt-secret \
  --from-literal=secret='your-jwt-secret-key' \
  --namespace=plugin-marketplace

# AWS secret
kubectl create secret generic aws-secret \
  --from-literal=access-key-id='your-access-key' \
  --from-literal=secret-access-key='your-secret-key' \
  --namespace=plugin-marketplace
```

#### 3. Create Deployment

`backend-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
  namespace: plugin-marketplace
spec:
  replicas: 3
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      containers:
      - name: backend
        image: registry.example.com/plugin-marketplace-backend:v1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
        - name: S3_BUCKET
          value: "plugin-marketplace"
        - name: AWS_ACCESS_KEY_ID
          valueFrom:
            secretKeyRef:
              name: aws-secret
              key: access-key-id
        - name: AWS_SECRET_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: aws-secret
              key: secret-access-key
        - name: AWS_REGION
          value: "us-east-1"
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: backend-service
  namespace: plugin-marketplace
spec:
  selector:
    app: backend
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

Deploy:

```bash
kubectl apply -f backend-deployment.yaml
```

## Frontend Deployment

### Option 1: Vercel (Recommended)

```bash
cd frontend

# Install Vercel CLI
npm i -g vercel

# Deploy
vercel --prod

# Set environment variables
vercel env add NEXT_PUBLIC_API_URL production
# Enter: https://api.plugins.gameengine.com
```

### Option 2: Docker Deployment

```dockerfile
# frontend/Dockerfile
FROM node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:20-alpine

WORKDIR /app

COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json

EXPOSE 3000

CMD ["npm", "start"]
```

Build and deploy:

```bash
docker build -t plugin-marketplace-frontend:v1.0.0 .
docker run -p 3000:3000 -e NEXT_PUBLIC_API_URL=https://api.plugins.gameengine.com plugin-marketplace-frontend:v1.0.0
```

### Option 3: Static Export

```bash
cd frontend

# Build static export
npm run build
npm run export

# Deploy to any static hosting service
# - Netlify: ./out directory
# - AWS S3 + CloudFront
# - GitHub Pages
```

## Storage Configuration

### AWS S3 Setup

```bash
# Create S3 bucket
aws s3 mb s3://plugin-marketplace --region us-east-1

# Enable versioning
aws s3api put-bucket-versioning \
  --bucket plugin-marketplace \
  --versioning-configuration Status=Enabled

# Set lifecycle policy
aws s3api put-bucket-lifecycle-configuration \
  --bucket plugin-marketplace \
  --lifecycle-configuration file://lifecycle.json

# lifecycle.json
{
  "Rules": [
    {
      "Id": "DeleteOldVersions",
      "Status": "Enabled",
      "Prefix": "plugins/",
      "NoncurrentVersionExpiration": { "NoncurrentDays": 30 }
    }
  ]
}

# Set CORS configuration
aws s3api put-bucket-cors \
  --bucket plugin-marketplace \
  --cors-configuration file://cors.json

# cors.json
{
  "CORSRules": [
    {
      "AllowedHeaders": ["*"],
      "AllowedMethods": ["GET", "HEAD"],
      "AllowedOrigins": ["https://plugins.gameengine.com"],
      "ExposeHeaders": ["ETag"]
    }
  ]
}
```

### CDN Configuration

```bash
# Create CloudFront distribution
aws cloudfront create-distribution \
  --distribution-config file://distribution.json

# distribution.json (simplified)
{
  "CallerReference": "plugin-marketplace-$(date +%s)",
  "Aliases": {
    "Items": ["cdn.plugins.gameengine.com"],
    "Quantity": 1
  },
  "DefaultCacheBehavior": {
    "TargetOriginId": "S3-plugin-marketplace",
    "ViewerProtocolPolicy": "redirect-to-https",
    "AllowedMethods": {
      "Items": ["GET", "HEAD"],
      "Quantity": 2
    },
    "ForwardedValues": {
      "QueryString": false,
      "Cookies": { "Forward": "none" }
    },
    "MinTTL": 86400,
    "Compress": true
  },
  "Origins": {
    "Items": [{
      "Id": "S3-plugin-marketplace",
      "DomainName": "plugin-marketplace.s3.amazonaws.com",
      "S3OriginConfig": {}
    }],
    "Quantity": 1
  },
  "Enabled": true,
  "PriceClass": "PriceClass_100"
}
```

## DNS and SSL

### DNS Configuration

```
# A Records
plugins.gameengine.com      →  Backend LoadBalancer IP
www.plugins.gameengine.com  →  Frontend LoadBalancer IP
cdn.plugins.gameengine.com  →  CloudFront domain

# CNAME
api.plugins.gameengine.com  →  plugins.gameengine.com
```

### SSL Certificates

#### Using Let's Encrypt with Certbot

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d plugins.gameengine.com -d www.plugins.gameengine.com

# Auto-renewal (configured automatically)
sudo certbot renew --dry-run
```

#### Using AWS Certificate Manager

```bash
# Request certificate
aws acm request-certificate \
  --domain-name plugins.gameengine.com \
  --subject-alternative-names www.plugins.gameengine.com \
  --validation-method DNS

# Validate via DNS
# Add CNAME records provided by AWS

# Wait for validation
aws acm wait certificate-validated \
  --certificate-arn arn:aws:acm:us-east-1:123456789012:certificate/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

## Monitoring and Logging

### Application Monitoring

#### Prometheus + Grafana

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'plugin-marketplace'
    static_configs:
      - targets: ['backend:8080']
```

```bash
# Run Prometheus
docker run -d \
  -p 9090:9090 \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus

# Run Grafana
docker run -d \
  -p 3001:3000 \
  grafana/grafana
```

### Log Aggregation

#### ELK Stack (Elasticsearch, Logstash, Kibana)

```yaml
# docker-compose.logging.yml
version: '3.8'

services:
  elasticsearch:
    image: elasticsearch:8.10.0
    environment:
      - discovery.type=single-node
      - "ES_JAVA_OPTS=-Xms512m -Xmx512m"
    ports:
      - "9200:9200"
    volumes:
      - esdata:/usr/share/elasticsearch/data

  logstash:
    image: logstash:8.10.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf
    ports:
      - "5044:5044"
    depends_on:
      - elasticsearch

  kibana:
    image: kibana:8.10.0
    ports:
      - "5601:5601"
    environment:
      - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
    depends_on:
      - elasticsearch

volumes:
  esdata:
```

### Error Tracking

#### Sentry Integration

```rust
// backend/Cargo.toml
[dependencies]
sentry = "0.31"

// src/main.rs
let _guard = sentry::init((
    std::env::var("SENTRY_DSN").unwrap(),
    sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    },
));
```

## Backup and Recovery

### Database Backup

#### Automated Backups

```bash
#!/bin/bash
# backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups"
DATABASE_URL="postgresql://admin:password@localhost:5432/plugin_marketplace"

# Create backup
pg_dump $DATABASE_URL > "$BACKUP_DIR/plugin_marketplace_$DATE.sql"

# Compress
gzip "$BACKUP_DIR/plugin_marketplace_$DATE.sql"

# Upload to S3
aws s3 cp "$BACKUP_DIR/plugin_marketplace_$DATE.sql.gz" \
  s3://plugin-marketplace-backups/

# Clean old backups (keep last 30 days)
find $BACKUP_DIR -name "plugin_marketplace_*.sql.gz" -mtime +30 -delete
```

Add to crontab:

```bash
# Daily backup at 2 AM
0 2 * * * /path/to/backup.sh
```

#### AWS RDS Automated Backups

```bash
# Enable automated backups
aws rds modify-db-instance \
  --db-instance-identifier plugin-marketplace-db \
  --backup-retention-period 7 \
  --preferred-backup-window 02:00-03:00 \
  --apply-immediately
```

### Disaster Recovery

#### Restore from Backup

```bash
# Stop application
docker-compose down

# Restore database
gunzip < plugin_marketplace_20240115_020000.sql.gz | \
  psql $DATABASE_URL

# Restart application
docker-compose up -d
```

#### Point-in-Time Recovery (RDS)

```bash
# Restore to specific time
aws rds restore-db-instance-to-point-in-time \
  --source-db-instance-identifier plugin-marketplace-db \
  --target-db-instance-identifier plugin-marketplace-db-restored \
  --restore-time 2024-01-15T10:00:00Z \
  --use-latest-restorable-time
```

## Performance Optimization

### Database Optimization

```sql
-- Create indexes for common queries
CREATE INDEX CONCURRENTLY idx_plugins_search ON plugins USING GIN(to_tsvector('english', name || ' ' || description));
CREATE INDEX CONCURRENTLY idx_plugins_rating ON plugins(rating_average DESC, rating_count DESC);
CREATE INDEX CONCURRENTLY idx_downloads_events_created ON download_events(created_at DESC);

-- Update statistics
ANALYZE plugins;
ANALYZE reviews;
ANALYZE download_events;
```

### Caching Strategy

```bash
# Redis cache for frequent queries
docker run -d \
  --name redis-cache \
  -p 6379:6379 \
  redis:7-alpine
```

```rust
// Implement caching in backend
use redis::AsyncCommands;

async fn get_plugin_with_cache(
    cache: &mut redis::aio::Connection,
    plugin_id: &str
) -> Result<Option<Plugin>, Error> {
    let cache_key = format!("plugin:{}", plugin_id);

    // Try cache first
    if let Ok(cached) = cache.get(&cache_key).await {
        if let Ok(plugin) = serde_json::from_str::<Plugin>(&cached) {
            return Ok(Some(plugin));
        }
    }

    // Cache miss - fetch from database
    let plugin = fetch_plugin_from_db(plugin_id).await?;

    // Store in cache (5 minute expiry)
    cache.set_ex(&cache_key, serde_json::to_string(&plugin)?, 300).await?;

    Ok(Some(plugin))
}
```

## Security Hardening

### Firewall Rules

```bash
# UFW rules
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable firewall
sudo ufw enable
```

### Rate Limiting

```rust
// Implement rate limiting
use governor::{Quota, RateLimiter};

const RATE_LIMIT: RateLimiter<
    &'static str,
    _, _
> = RateLimiter::direct(
    Quota::per_second(NonZeroU32::new(10))
);

async fn rate_limit_middleware(
    req: ServiceRequest,
    next: Next<impl Message>
) -> Result<ServiceResponse, Error> {
    // Check rate limit
    if !RATE_LIMIT.check() {
        return Ok(ServiceResponse::new(
            req.into_response(),
            HttpResponse::TooManyRequests().finish()
        ));
    }

    next.call(req).await
}
```

## Scaling Strategy

### Horizontal Scaling

```bash
# Scale backend deployment
kubectl scale deployment backend --replicas=10 -n plugin-marketplace

# Configure Horizontal Pod Autoscaler
kubectl autoscale deployment backend \
  --cpu-percent=70 \
  --min=3 \
  --max=20 \
  --namespace=plugin-marketplace
```

### Database Scaling

```bash
# Read replicas for better performance
aws rds create-db-instance-read-replica \
  --db-instance-identifier plugin-marketplace-db-replica-1 \
  --source-db-instance-identifier plugin-marketplace-db
```

## Troubleshooting

### Common Issues

#### Database Connection Failed

```bash
# Check database status
kubectl get pods -n plugin-marketplace

# View logs
kubectl logs -f deployment/backend -n plugin-marketplace

# Test connection
psql $DATABASE_URL
```

#### High Memory Usage

```bash
# Check resource usage
kubectl top pods -n plugin-marketplace

# Adjust limits
kubectl set resources deployment backend \
  --limits=memory=1Gi \
  --requests=memory=512Mi \
  -n plugin-marketplace
```

#### Slow API Responses

```bash
# Check database query performance
SELECT * FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;

# Add missing indexes
CREATE INDEX CONCURRENTLY idx_missing ON table(column);
```

## Maintenance

### Regular Maintenance Tasks

```bash
#!/bin/bash
# maintenance.sh

# Update dependencies
cd backend && cargo update
cd ../frontend && npm update

# Run database migrations
sqlx migrate run

# Clear old logs
find /var/log -name "*.log" -mtime +30 -delete

# Backup before maintenance
./backup.sh

# Restart services
kubectl rollout restart deployment/backend -n plugin-marketplace
kubectl rollout restart deployment/frontend -n plugin-marketplace
```

## Cost Optimization

### AWS Cost Saving Tips

```bash
# Use Reserved Instances for production
# Enable S3 Intelligent-Tiering
aws s3api put-bucket-versioning \
  --bucket plugin-marketplace \
  --versioning-configuration Status=Enabled

# Use CloudFront for lower data transfer costs
# Set up lifecycle policies for old data
```

This deployment guide covers all aspects of deploying the plugin marketplace to production. Adjust configurations based on your specific requirements and infrastructure.
