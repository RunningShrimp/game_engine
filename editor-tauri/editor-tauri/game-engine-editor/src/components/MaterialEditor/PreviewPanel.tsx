/**
 * Preview Panel Component
 * Displays a real-time preview of the material on different meshes
 */

import React, { useRef, useEffect, useState } from 'react';
import { Material } from '../../types/material';
import './PreviewPanel.css';

interface PreviewPanelProps {
  material: Material;
}

export const PreviewPanel: React.FC<PreviewPanelProps> = ({ material }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [meshType, setMeshType] = useState<'sphere' | 'cube' | 'plane'>('sphere');
  const [rotation, setRotation] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // WebGL context
  const glRef = useRef<WebGLRenderingContext | null>(null);
  const programRef = useRef<WebGLProgram | null>(null);
  const animationFrameRef = useRef<number | undefined>(undefined);

  // Initialize WebGL
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const gl = (canvas.getContext('webgl2') || canvas.getContext('webgl')) as WebGLRenderingContext | null;
    if (!gl) {
      console.error('WebGL not supported');
      return;
    }

    glRef.current = gl;

    // Set clear color
    gl.clearColor(0.1, 0.1, 0.1, 1.0);
    gl.enable(gl.DEPTH_TEST);
    gl.enable(gl.CULL_FACE);

    // Initialize shader program
    initShaderProgram(gl);

    // Start render loop
    const render = () => {
      drawScene(gl);
      animationFrameRef.current = requestAnimationFrame(render);
    };
    render();

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Update material uniform
  useEffect(() => {
    if (glRef.current && programRef.current) {
      updateMaterialUniforms();
    }
  }, [material]);

  // Initialize shader program
  const initShaderProgram = (gl: WebGLRenderingContext) => {
    // Vertex shader
    const vsSource = `
      attribute vec4 aVertexPosition;
      attribute vec3 aVertexNormal;

      uniform mat4 uModelMatrix;
      uniform mat4 uViewMatrix;
      uniform mat4 uProjectionMatrix;
      uniform mat4 uNormalMatrix;

      varying highp vec3 vNormal;
      varying highp vec3 vPosition;

      void main(void) {
        vPosition = (uModelMatrix * aVertexPosition).xyz;
        vNormal = (uNormalMatrix * vec4(aVertexNormal, 1.0)).xyz;
        gl_Position = uProjectionMatrix * uViewMatrix * uModelMatrix * aVertexPosition;
      }
    `;

    // Fragment shader (simplified PBR)
    const fsSource = `
      precision highp float;

      varying highp vec3 vNormal;
      varying highp vec3 vPosition;

      uniform vec3 uBaseColor;
      uniform float uMetallic;
      uniform float uRoughness;
      uniform float uAO;

      const vec3 lightPosition = vec3(5.0, 5.0, 5.0);
      const vec3 lightColor = vec3(1.0, 1.0, 1.0);

      void main(void) {
        vec3 N = normalize(vNormal);
        vec3 L = normalize(lightPosition - vPosition);
        vec3 V = normalize(-vPosition);
        vec3 H = normalize(L + V);

        // Simple diffuse lighting
        float diff = max(dot(N, L), 0.0);

        // Simple specular
        float spec = pow(max(dot(N, H), 0.0), 32.0 * (1.0 - uRoughness));

        // Combine
        vec3 ambient = 0.1 * uBaseColor * uAO;
        vec3 diffuse = diff * uBaseColor;
        vec3 specular = spec * lightColor * mix(0.04, uBaseColor, uMetallic);

        vec3 color = ambient + diffuse * (1.0 - uMetallic) + specular;

        gl_FragColor = vec4(color, 1.0);
      }
    `;

    const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource);
    const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource);

    if (!vertexShader || !fragmentShader) return;

    const shaderProgram = gl.createProgram();
    if (!shaderProgram) return;

    gl.attachShader(shaderProgram, vertexShader);
    gl.attachShader(shaderProgram, fragmentShader);
    gl.linkProgram(shaderProgram);

    if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
      console.error('Unable to initialize the shader program:', gl.getProgramInfoLog(shaderProgram));
      return;
    }

    programRef.current = shaderProgram;
  };

  // Load shader
  const loadShader = (gl: WebGLRenderingContext, type: number, source: string) => {
    const shader = gl.createShader(type);
    if (!shader) return null;

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error('An error occurred compiling the shaders:', gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  };

  // Draw scene
  const drawScene = (gl: WebGLRenderingContext) => {
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    if (!programRef.current) return;

    // Get mesh geometry
    const { positions, normals, indices } = getMeshGeometry(meshType);

    // Create buffers
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

    const normalBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, normalBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(normals), gl.STATIC_DRAW);

    const indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW);

    // Set up attributes
    const vertexPosition = gl.getAttribLocation(programRef.current, 'aVertexPosition');
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.vertexAttribPointer(vertexPosition, 3, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(vertexPosition);

    const vertexNormal = gl.getAttribLocation(programRef.current, 'aVertexNormal');
    gl.bindBuffer(gl.ARRAY_BUFFER, normalBuffer);
    gl.vertexAttribPointer(vertexNormal, 3, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(vertexNormal);

    // Set up matrices
    const projectionMatrix = createPerspectiveMatrix(45, 1, 0.1, 100);
    const viewMatrix = createLookAtMatrix([0, 0, 3], [0, 0, 0], [0, 1, 0]);
    const modelMatrix = createRotationMatrix(rotation.x, rotation.y);
    const normalMatrix = modelMatrix; // Simplified

    // Set uniforms
    gl.useProgram(programRef.current);

    gl.uniformMatrix4fv(
      gl.getUniformLocation(programRef.current, 'uProjectionMatrix'),
      false,
      projectionMatrix
    );
    gl.uniformMatrix4fv(
      gl.getUniformLocation(programRef.current, 'uViewMatrix'),
      false,
      viewMatrix
    );
    gl.uniformMatrix4fv(
      gl.getUniformLocation(programRef.current, 'uModelMatrix'),
      false,
      modelMatrix
    );
    gl.uniformMatrix4fv(
      gl.getUniformLocation(programRef.current, 'uNormalMatrix'),
      false,
      normalMatrix
    );

    // Update material uniforms
    updateMaterialUniforms();

    // Draw
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.drawElements(gl.TRIANGLES, indices.length, gl.UNSIGNED_SHORT, 0);

    // Cleanup
    gl.deleteBuffer(positionBuffer);
    gl.deleteBuffer(normalBuffer);
    gl.deleteBuffer(indexBuffer);
  };

  // Update material uniforms
  const updateMaterialUniforms = () => {
    if (!glRef.current || !programRef.current) return;

    const gl = glRef.current;

    // Get material values from PBRMaster node
    const pbrMaster = material.nodes.find(n => n.type === 'pbr_master');
    if (pbrMaster) {
      const baseColorParam = pbrMaster.parameters.find(p => p.id === 'baseColor');
      const metallicParam = pbrMaster.parameters.find(p => p.id === 'metallic');
      const roughnessParam = pbrMaster.parameters.find(p => p.id === 'roughness');
      const aoParam = pbrMaster.parameters.find(p => p.id === 'ao');

      const baseColor = Array.isArray(baseColorParam?.value)
        ? baseColorParam.value
        : [1, 1, 1, 1];

      gl.uniform3f(
        gl.getUniformLocation(programRef.current, 'uBaseColor'),
        baseColor[0], baseColor[1], baseColor[2]
      );
      gl.uniform1f(
        gl.getUniformLocation(programRef.current, 'uMetallic'),
        metallicParam?.value ?? 0
      );
      gl.uniform1f(
        gl.getUniformLocation(programRef.current, 'uRoughness'),
        roughnessParam?.value ?? 0.5
      );
      gl.uniform1f(
        gl.getUniformLocation(programRef.current, 'uAO'),
        aoParam?.value ?? 1
      );
    } else {
      // Default values
      gl.uniform3f(gl.getUniformLocation(programRef.current, 'uBaseColor'), 1, 1, 1);
      gl.uniform1f(gl.getUniformLocation(programRef.current, 'uMetallic'), 0);
      gl.uniform1f(gl.getUniformLocation(programRef.current, 'uRoughness'), 0.5);
      gl.uniform1f(gl.getUniformLocation(programRef.current, 'uAO'), 1);
    }
  };

  // Get mesh geometry
  const getMeshGeometry = (type: 'sphere' | 'cube' | 'plane') => {
    // Simplified geometry generation
    if (type === 'cube') {
      return createCubeGeometry();
    } else if (type === 'plane') {
      return createPlaneGeometry();
    } else {
      return createSphereGeometry();
    }
  };

  // Handle mouse rotation
  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setDragStart({ x: e.clientX, y: e.clientY });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return;

    const deltaX = e.clientX - dragStart.x;
    const deltaY = e.clientY - dragStart.y;

    setRotation(prev => ({
      x: prev.x + deltaY * 0.01,
      y: prev.y + deltaX * 0.01,
    }));

    setDragStart({ x: e.clientX, y: e.clientY });
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  return (
    <div className="preview-panel">
      <div className="preview-header">
        <h4>Preview</h4>
      </div>

      <div className="preview-controls">
        <button
          className={meshType === 'sphere' ? 'active' : ''}
          onClick={() => setMeshType('sphere')}
        >
          Sphere
        </button>
        <button
          className={meshType === 'cube' ? 'active' : ''}
          onClick={() => setMeshType('cube')}
        >
          Cube
        </button>
        <button
          className={meshType === 'plane' ? 'active' : ''}
          onClick={() => setMeshType('plane')}
        >
          Plane
        </button>
      </div>

      <canvas
        ref={canvasRef}
        width={300}
        height={300}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />

      <div className="preview-info">
        <p>Drag to rotate</p>
      </div>
    </div>
  );
};

// Helper functions for geometry and matrix creation

function createSphereGeometry() {
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];

  const latitudeBands = 30;
  const longitudeBands = 30;
  const radius = 1;

  for (let lat = 0; lat <= latitudeBands; lat++) {
    const theta = (lat * Math.PI) / latitudeBands;
    const sinTheta = Math.sin(theta);
    const cosTheta = Math.cos(theta);

    for (let long = 0; long <= longitudeBands; long++) {
      const phi = (long * 2 * Math.PI) / longitudeBands;
      const sinPhi = Math.sin(phi);
      const cosPhi = Math.cos(phi);

      const x = cosPhi * sinTheta;
      const y = cosTheta;
      const z = sinPhi * sinTheta;

      normals.push(x, y, z);
      positions.push(radius * x, radius * y, radius * z);
    }
  }

  for (let lat = 0; lat < latitudeBands; lat++) {
    for (let long = 0; long < longitudeBands; long++) {
      const first = lat * (longitudeBands + 1) + long;
      const second = first + longitudeBands + 1;

      indices.push(first, second, first + 1);
      indices.push(second, second + 1, first + 1);
    }
  }

  return { positions, normals, indices };
}

function createCubeGeometry() {
  const positions = [
    // Front
    -1, -1, 1, 1, -1, 1, 1, 1, 1, -1, 1, 1,
    // Back
    -1, -1, -1, -1, 1, -1, 1, 1, -1, 1, -1, -1,
    // Top
    -1, 1, -1, -1, 1, 1, 1, 1, 1, 1, 1, -1,
    // Bottom
    -1, -1, -1, 1, -1, -1, 1, -1, 1, -1, -1, 1,
    // Right
    1, -1, -1, 1, 1, -1, 1, 1, 1, 1, -1, 1,
    // Left
    -1, -1, -1, -1, -1, 1, -1, 1, 1, -1, 1, -1,
  ];

  const normals = [
    // Front
    0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1,
    // Back
    0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1,
    // Top
    0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
    // Bottom
    0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0,
    // Right
    1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0,
    // Left
    -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0,
  ];

  const indices = [
    0, 1, 2, 0, 2, 3, // Front
    4, 5, 6, 4, 6, 7, // Back
    8, 9, 10, 8, 10, 11, // Top
    12, 13, 14, 12, 14, 15, // Bottom
    16, 17, 18, 16, 18, 19, // Right
    20, 21, 22, 20, 22, 23, // Left
  ];

  return { positions, normals, indices };
}

function createPlaneGeometry() {
  const size = 2;
  const positions = [
    -size, 0, -size,
    size, 0, -size,
    size, 0, size,
    -size, 0, size,
  ];

  const normals = [
    0, 1, 0,
    0, 1, 0,
    0, 1, 0,
    0, 1, 0,
  ];

  const indices = [0, 1, 2, 0, 2, 3];

  return { positions, normals, indices };
}

function createPerspectiveMatrix(fov: number, aspect: number, near: number, far: number) {
  const f = 1.0 / Math.tan((fov * Math.PI) / 360);
  const nf = 1 / (near - far);

  return new Float32Array([
    f / aspect, 0, 0, 0,
    0, f, 0, 0,
    0, 0, (far + near) * nf, -1,
    0, 0, 2 * far * near * nf, 0,
  ]);
}

function createLookAtMatrix(eye: number[], center: number[], up: number[]) {
  const zAxis = normalize([eye[0] - center[0], eye[1] - center[1], eye[2] - center[2]]);
  const xAxis = normalize(cross(up, zAxis));
  const yAxis = cross(zAxis, xAxis);

  return new Float32Array([
    xAxis[0], yAxis[0], zAxis[0], 0,
    xAxis[1], yAxis[1], zAxis[1], 0,
    xAxis[2], yAxis[2], zAxis[2], 0,
    -dot(xAxis, eye), -dot(yAxis, eye), -dot(zAxis, eye), 1,
  ]);
}

function createRotationMatrix(x: number, y: number) {
  const cosX = Math.cos(x);
  const sinX = Math.sin(x);
  const cosY = Math.cos(y);
  const sinY = Math.sin(y);

  return new Float32Array([
    cosY, 0, sinY, 0,
    sinX * sinY, cosX, -sinX * cosY, 0,
    -cosX * sinY, sinX, cosX * cosY, 0,
    0, 0, 0, 1,
  ]);
}

function normalize(v: number[]) {
  const len = Math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
  return [v[0] / len, v[1] / len, v[2] / len];
}

function cross(a: number[], b: number[]) {
  return [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ];
}

function dot(a: number[], b: number[]) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

export default PreviewPanel;
