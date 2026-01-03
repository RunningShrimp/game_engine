// Ray casting and 3D picking utilities

import { Vector3, Matrix4 } from './math3d';

export class Ray {
  constructor(
    public origin: Vector3,
    public direction: Vector3
  ) {
    this.direction = direction.normalize();
  }

  at(t: number): Vector3 {
    return this.origin.add(this.direction.multiply(t));
  }

  distanceToPoint(point: Vector3): number {
    const v = point.subtract(this.origin);
    const projection = v.dot(this.direction);
    const closestPoint = this.origin.add(this.direction.multiply(projection));
    return point.distanceTo(closestPoint);
  }
}

export class Camera {
  constructor(
    public position: Vector3,
    public target: Vector3,
    public fov: number = 45,
    public aspect: number = 1,
    public near: number = 0.1,
    public far: number = 1000
  ) {}

  getViewMatrix(): Matrix4 {
    const forward = this.target.subtract(this.position).normalize();
    const up = Vector3.up;

    const right = forward.cross(up).normalize();
    const newUp = right.cross(forward).normalize();

    const viewMatrix = new Matrix4();
    viewMatrix['elements'] = [
      right.x, newUp.x, -forward.x, 0,
      right.y, newUp.y, -forward.y, 0,
      right.z, newUp.z, -forward.z, 0,
      -right.dot(this.position), -newUp.dot(this.position), forward.dot(this.position), 1
    ];

    return viewMatrix;
  }

  getProjectionMatrix(): Matrix4 {
    const fovRad = (this.fov * Math.PI) / 180;
    const f = 1.0 / Math.tan(fovRad / 2);

    const projMatrix = new Matrix4();
    projMatrix['elements'] = [
      f / this.aspect, 0, 0, 0,
      0, f, 0, 0,
      0, 0, (this.far + this.near) / (this.near - this.far), -1,
      0, 0, (2 * this.far * this.near) / (this.near - this.far), 0
    ];

    return projMatrix;
  }

  screenPointToRay(screenX: number, screenY: number, viewportWidth: number, viewportHeight: number): Ray {
    // Normalize to [-1, 1]
    const ndc = {
      x: (screenX / viewportWidth) * 2 - 1,
      y: -(screenY / viewportHeight) * 2 + 1
    };

    // Calculate ray direction in clip space
    const clipNear = new Vector3(ndc.x, ndc.y, -1);
    const clipFar = new Vector3(ndc.x, ndc.y, 1);

    const invProj = this.getProjectionMatrix().invert();
    const invView = this.getViewMatrix().invert();

    // Transform to world space
    const eyeNear = invProj!.multiplyVector3(clipNear);
    const eyeFar = invProj!.multiplyVector3(clipFar);

    const worldNear = invView!.multiplyVector3(eyeNear);
    const worldFar = invView!.multiplyVector3(eyeFar);

    const direction = worldFar.subtract(worldNear).normalize();

    return new Ray(worldNear, direction);
  }

  worldPointToScreen(worldPos: Vector3, viewportWidth: number, viewportHeight: number): { x: number; y: number } {
    const viewMatrix = this.getViewMatrix();
    const projMatrix = this.getProjectionMatrix();

    // Transform to clip space
    const viewPos = viewMatrix.multiplyVector3(worldPos);
    const clipPos = projMatrix.multiplyVector3(viewPos);

    // Perspective divide
    const ndc = {
      x: clipPos.x / clipPos.z,
      y: clipPos.y / clipPos.z
    };

    // Convert to screen space
    return {
      x: (ndc.x + 1) * 0.5 * viewportWidth,
      y: (1 - ndc.y) * 0.5 * viewportHeight
    };
  }
}

export class Plane {
  constructor(
    public normal: Vector3,
    public distance: number
  ) {}

  static fromPointNormal(point: Vector3, normal: Vector3): Plane {
    return new Plane(normal.normalize(), -normal.dot(point));
  }

  intersectRay(ray: Ray): Vector3 | null {
    const denom = this.normal.dot(ray.direction);

    // Ray is parallel to plane
    if (Math.abs(denom) < 0.0001) {
      return null;
    }

    const t = -(this.normal.dot(ray.origin) + this.distance) / denom;

    if (t < 0) {
      return null;
    }

    return ray.at(t);
  }
}

export interface BoundingBox {
  min: Vector3;
  max: Vector3;
}

export class BoundingBoxHelper {
  static fromCenterSize(center: Vector3, size: Vector3): BoundingBox {
    const half = size.multiply(0.5);
    return {
      min: center.subtract(half),
      max: center.add(half)
    };
  }

  static intersectRay(ray: Ray, box: BoundingBox): boolean {
    const min = box.min;
    const max = box.max;

    const t1 = (min.x - ray.origin.x) / ray.direction.x;
    const t2 = (max.x - ray.origin.x) / ray.direction.x;
    const t3 = (min.y - ray.origin.y) / ray.direction.y;
    const t4 = (max.y - ray.origin.y) / ray.direction.y;
    const t5 = (min.z - ray.origin.z) / ray.direction.z;
    const t6 = (max.z - ray.origin.z) / ray.direction.z;

    const tmin = Math.max(
      Math.max(Math.min(t1, t2), Math.min(t3, t4)),
      Math.min(t5, t6)
    );
    const tmax = Math.min(
      Math.min(Math.max(t1, t2), Math.max(t3, t4)),
      Math.max(t5, t6)
    );

    return tmax >= 0 && tmin <= tmax;
  }
}

export interface Sphere {
  center: Vector3;
  radius: number;
}

export class SphereHelper {
  static intersectRay(ray: Ray, sphere: Sphere): boolean {
    const oc = ray.origin.subtract(sphere.center);
    const a = ray.direction.dot(ray.direction);
    const b = 2 * oc.dot(ray.direction);
    const c = oc.dot(oc) - sphere.radius * sphere.radius;

    const discriminant = b * b - 4 * a * c;

    return discriminant >= 0;
  }

  static intersectRayDistance(ray: Ray, sphere: Sphere): number[] {
    const oc = ray.origin.subtract(sphere.center);
    const a = ray.direction.dot(ray.direction);
    const b = 2 * oc.dot(ray.direction);
    const c = oc.dot(oc) - sphere.radius * sphere.radius;

    const discriminant = b * b - 4 * a * c;

    if (discriminant < 0) {
      return [];
    }

    const sqrtDiscriminant = Math.sqrt(discriminant);
    const t1 = (-b - sqrtDiscriminant) / (2 * a);
    const t2 = (-b + sqrtDiscriminant) / (2 * a);

    if (t1 >= 0 && t2 >= 0) {
      return [Math.min(t1, t2), Math.max(t1, t2)];
    } else if (t1 >= 0) {
      return [t1];
    } else if (t2 >= 0) {
      return [t2];
    }

    return [];
  }
}

// Line segment intersection for gizmo axes
export interface LineSegment {
  start: Vector3;
  end: Vector3;
  thickness: number;
}

export class LineHelper {
  static intersectRay(ray: Ray, segment: LineSegment): { distance: number; point: Vector3 } | null {
    const lineDir = segment.end.subtract(segment.start).normalize();
    const lineToRay = ray.origin.subtract(segment.start);

    const a = ray.direction.dot(ray.direction);
    const b = ray.direction.dot(lineDir);
    const c = lineDir.dot(lineDir);
    const d = ray.direction.dot(lineToRay);
    const e = lineDir.dot(lineToRay);

    const denom = a * c - b * b;

    if (Math.abs(denom) < 0.0001) {
      return null;
    }

    let t = (b * e - c * d) / denom;
    let s = (a * e - b * d) / denom;

    // Check if intersection is within line segment
    if (s >= 0 && s <= 1 && t >= 0) {
      const point = ray.at(t);
      const distance = ray.distanceToPoint(point);

      // Check if within thickness threshold
      if (distance <= segment.thickness) {
        return { distance: t, point };
      }
    }

    return null;
  }
}

// Circle intersection for rotation gizmo
export interface Circle {
  center: Vector3;
  normal: Vector3;
  radius: number;
  thickness: number;
}

export class CircleHelper {
  static intersectRay(ray: Ray, circle: Circle): { distance: number; point: Vector3 } | null {
    const plane = Plane.fromPointNormal(circle.center, circle.normal);
    const intersection = plane.intersectRay(ray);

    if (!intersection) {
      return null;
    }

    const distFromCenter = intersection.distanceTo(circle.center);
    const distFromRing = Math.abs(distFromCenter - circle.radius);

    if (distFromRing <= circle.thickness) {
      const distance = intersection.distanceTo(ray.origin);
      return { distance, point: intersection };
    }

    return null;
  }
}
