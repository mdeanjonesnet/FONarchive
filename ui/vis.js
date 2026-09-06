const VS = `#version 300 es
in vec2 aXZ;
in vec2 aUV;
uniform mat4 uMVP;
uniform float uTime;
uniform float uSpeed;
out vec2 vUV;
out vec3 vWorld;
out vec3 vNrm;

float height(vec2 p, float t) {
  float x = p.x;
  float z = p.y;
  float wx = x + 0.65 * sin(z * 0.29 + t * 0.11);
  float wz = z + 0.45 * sin(x * 0.25 - t * 0.08);
  float n = 0.0;
  n += 0.95 * abs(sin(wx * 0.37 + t * 0.26)) * abs(cos(wz * 0.21 + 1.1));
  n += 0.72 * abs(sin(wx * 0.71 - t * 0.17 + 2.4)) * abs(cos(wz * 0.44 + t * 0.13));
  n += 0.40 * abs(sin(wx * 1.48 + wz * 0.78 + t * 0.33));
  n += 0.34 * abs(sin(wx * 0.18 + 0.9) * cos(wz * 0.13 + 2.2));
  n += 0.42 * abs(sin(wx * 0.53 + 1.7 + t * 0.09));
  n += 0.20 * sin(wx * 2.20 - wz * 1.35 + t * 0.22);
  n = pow(max(n, 0.0), 1.20);
  float base = 0.45 + 0.40 * sin(x * 0.15 + 0.5) + 0.28 * sin(x * 0.07 - 1.3);
  float near = smoothstep(0.4, 10.0, z);
  float roll = 0.10 * sin(x * 1.55 + t * 0.52) * sin(z * 1.18 + t * 0.40);
  return mix(0.22 + roll, n * 1.85 + base, near);
}

void main() {
  float t = uTime;
  vec2 p = aXZ;
  float hs = 0.22;
  float h  = height(p, t);
  float hL = height(p + vec2(-hs, 0.0), t);
  float hR = height(p + vec2( hs, 0.0), t);
  float hD = height(p + vec2(0.0, -hs), t);
  float hU = height(p + vec2(0.0,  hs), t);
  vec3 nrm = normalize(vec3(hL - hR, 2.0 * hs, hD - hU));
  vec3 world = vec3(p.x, h, p.y);
  vWorld = world;
  vNrm = nrm;
  vUV = aUV;
  gl_Position = uMVP * vec4(world, 1.0);
}`;

const FS = `#version 300 es
precision highp float;
in vec2 vUV;
in vec3 vWorld;
in vec3 vNrm;
uniform vec3 uAccent;
uniform vec3 uBg;
uniform vec3 uCam;
out vec4 fragColor;

void main() {
  float e = min(min(vUV.x, vUV.y), min(1.0 - vUV.x, 1.0 - vUV.y));
  float fw = fwidth(e);
  float line = 1.0 - smoothstep(fw * 0.35, fw * 1.25, e);
  float ndl = clamp(dot(normalize(vNrm), normalize(vec3(0.35, 1.0, 0.4))), 0.0, 1.0);
  vec3 fill = mix(uBg, uAccent, 0.02 + 0.08 * ndl);
  float dist = length(vWorld - uCam);
  float fog = smoothstep(12.0, 40.0, dist);
  vec3 col = mix(fill, uAccent, line);
  col = mix(col, uBg, fog);
  col += uAccent * fog * 0.06;
  col += uAccent * line * (1.0 - fog) * 0.12;
  fragColor = vec4(col, 1.0);
}`;

export const IDLE_SPEED = 0.7;
export const RUN_SPEED = IDLE_SPEED * 3;

function compile(gl, type, src) {
  const s = gl.createShader(type);
  gl.shaderSource(s, src);
  gl.compileShader(s);
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
    throw new Error(gl.getShaderInfoLog(s));
  }
  return s;
}

function mul(a, b) {
  const o = new Float32Array(16);
  for (let c = 0; c < 4; c++) {
    for (let r = 0; r < 4; r++) {
      o[c * 4 + r] =
        a[0 * 4 + r] * b[c * 4 + 0] +
        a[1 * 4 + r] * b[c * 4 + 1] +
        a[2 * 4 + r] * b[c * 4 + 2] +
        a[3 * 4 + r] * b[c * 4 + 3];
    }
  }
  return o;
}

function perspective(fovy, aspect, near, far) {
  const f = 1 / Math.tan(fovy / 2);
  const m = new Float32Array(16);
  m[0] = f / aspect;
  m[5] = f;
  m[10] = (far + near) / (near - far);
  m[11] = -1;
  m[14] = (2 * far * near) / (near - far);
  return m;
}

function lookAt(eye, target, up) {
  const z = norm(sub(eye, target));
  const x = norm(cross(up, z));
  const y = cross(z, x);
  const m = new Float32Array(16);
  m[0] = x[0];
  m[1] = y[0];
  m[2] = z[0];
  m[4] = x[1];
  m[5] = y[1];
  m[6] = z[1];
  m[8] = x[2];
  m[9] = y[2];
  m[10] = z[2];
  m[12] = -dot(x, eye);
  m[13] = -dot(y, eye);
  m[14] = -dot(z, eye);
  m[15] = 1;
  return m;
}

function sub(a, b) {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}
function dot(a, b) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}
function cross(a, b) {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}
function norm(v) {
  const l = Math.hypot(v[0], v[1], v[2]) || 1;
  return [v[0] / l, v[1] / l, v[2] / l];
}

export function hexToRgb(hex) {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

export function createVis(canvas) {
  const gl = canvas.getContext("webgl2", {
    antialias: true,
    alpha: false,
    preserveDrawingBuffer: false,
  });
  if (!gl) {
    return {
      setSpeed() {},
      setAccent() {},
      setBg() {},
      resize() {},
    };
  }

  const prog = gl.createProgram();
  gl.attachShader(prog, compile(gl, gl.VERTEX_SHADER, VS));
  gl.attachShader(prog, compile(gl, gl.FRAGMENT_SHADER, FS));
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(prog));
  }
  gl.useProgram(prog);
  const vao = gl.createVertexArray();
  gl.bindVertexArray(vao);

  const COLS = 52;
  const ROWS = 40;
  const X0 = -14.0,
    X1 = 14.0;
  const Z0 = 0.4,
    Z1 = 44.0;
  const quads = COLS * ROWS;
  const verts = quads * 6;
  const xz = new Float32Array(verts * 2);
  const uv = new Float32Array(verts * 2);
  let w = 0;
  function emitVert(x, z, u, v) {
    xz[w * 2] = x;
    xz[w * 2 + 1] = z;
    uv[w * 2] = u;
    uv[w * 2 + 1] = v;
    w++;
  }
  for (let j = 0; j < ROWS; j++) {
    const z0 = Z0 + (Z1 - Z0) * (j / ROWS);
    const z1 = Z0 + (Z1 - Z0) * ((j + 1) / ROWS);
    for (let i = 0; i < COLS; i++) {
      const x0 = X0 + (X1 - X0) * (i / COLS);
      const x1 = X0 + (X1 - X0) * ((i + 1) / COLS);
      emitVert(x0, z0, 0, 0);
      emitVert(x1, z0, 1, 0);
      emitVert(x0, z1, 0, 1);
      emitVert(x1, z0, 1, 0);
      emitVert(x1, z1, 1, 1);
      emitVert(x0, z1, 0, 1);
    }
  }

  function buf(data, loc, size) {
    const b = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, b);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, size, gl.FLOAT, false, 0, 0);
  }
  buf(xz, gl.getAttribLocation(prog, "aXZ"), 2);
  buf(uv, gl.getAttribLocation(prog, "aUV"), 2);

  const uMVP = gl.getUniformLocation(prog, "uMVP");
  const uTime = gl.getUniformLocation(prog, "uTime");
  const uSpeed = gl.getUniformLocation(prog, "uSpeed");
  const uAccent = gl.getUniformLocation(prog, "uAccent");
  const uBg = gl.getUniformLocation(prog, "uBg");
  const uCam = gl.getUniformLocation(prog, "uCam");

  // A little further back than look-dev so more of the range sits in frame.
  const eye = [0.28, 3.5, -4.3];
  const target = [0.08, 1.05, 20.5];
  let speed = IDLE_SPEED;
  let speedTarget = IDLE_SPEED;
  let phase = 0;
  let lastT = 0;
  let accent = hexToRgb("#ffb000");
  let bg = hexToRgb("#000000");

  gl.enable(gl.DEPTH_TEST);
  gl.depthFunc(gl.LEQUAL);
  gl.clearColor(bg[0], bg[1], bg[2], 1);

  function resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const wCss = canvas.clientWidth || 248;
    const hCss = canvas.clientHeight || 240;
    canvas.width = Math.max(8, Math.round(wCss * dpr));
    canvas.height = Math.max(8, Math.round(hCss * dpr));
    gl.viewport(0, 0, canvas.width, canvas.height);
  }

  const t0 = performance.now();
  function frame(now) {
    const t = (now - t0) / 1000;
    const dt = Math.min(0.05, Math.max(0, t - lastT));
    lastT = t;
    // ~0.3s ease so idle ↔ running doesn't pop.
    const k = 1 - Math.exp(-dt / 0.32);
    speed += (speedTarget - speed) * k;
    phase += dt * speed;
    const aspect = canvas.width / Math.max(canvas.height, 1);
    const mvp = mul(
      perspective((44 * Math.PI) / 180, aspect, 0.25, 70),
      lookAt(eye, target, [0, 1, 0])
    );
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.uniformMatrix4fv(uMVP, false, mvp);
    gl.uniform1f(uTime, phase);
    gl.uniform1f(uSpeed, 1.0);
    gl.uniform3fv(uAccent, accent);
    gl.uniform3fv(uBg, bg);
    gl.uniform3fv(uCam, eye);
    gl.drawArrays(gl.TRIANGLES, 0, verts);
    requestAnimationFrame(frame);
  }

  resize();
  requestAnimationFrame(frame);

  return {
    setSpeed(next) {
      speedTarget = next;
    },
    setAccent(hex) {
      accent = hexToRgb(hex);
    },
    setBg(hex) {
      bg = hexToRgb(hex);
      gl.clearColor(bg[0], bg[1], bg[2], 1);
    },
    resize,
  };
}
