// Hand-written JS glue for the wilhelm_renderer browser backend.
//
// Supplies the "wilhelm" wasm import module consumed by
// wilhelm_renderer_sys/src/web/. Owns the WebGL2 context and the
// integer-id -> GL-object handle tables (the C-shaped contract names GL
// objects with integers; WebGL uses opaque objects). Id 0 maps to null,
// matching GL unbind semantics.
//
// Shader dialect: sources arrive as the crate's #version 330 core GLSL;
// this backend rewrites the header to #version 300 es and injects a
// default float precision into fragment shaders. (Interim for the spike —
// the decided end state authors shaders in 300 es and has the NATIVE
// backend rewrite instead; see docs/DESIGN_WASM.md item 3.)
"use strict";

(function () {
  const canvas = document.getElementById("wilhelm-canvas");
  const utf8 = new TextDecoder();

  // Fullscreen: the canvas is the "monitor". Size it to the browser window
  // before the module starts so _glfwCreateFullscreenWindow sees the real
  // dimensions; keep it matched on window resize (dispatched to the
  // engine's GLFW-style callbacks via wilhelm_dispatch_resize).
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;

  let gl = null;
  let memory = null;

  const objs = [null]; // shaders, programs, buffers, VAOs, textures
  const uniforms = [null]; // uniform locations
  const uniformCache = new Map(); // "program:name" -> uniforms index (or -1)
  const shaderTypes = new Map(); // shader id -> type (for precision injection)

  const GL_MULTISAMPLE = 0x809d;
  const GL_FRAGMENT_SHADER = 0x8b30;
  const GL_RED = 0x1903;
  const GL_RGB = 0x1907;

  const bytes = () => new Uint8Array(memory.buffer);
  const str = (ptr, len) => utf8.decode(new Uint8Array(memory.buffer, ptr, len));
  const f32 = (ptr, n) => new Float32Array(memory.buffer, ptr, n);
  const i32 = (ptr, n) => new Int32Array(memory.buffer, ptr, n);

  function alloc(obj) {
    objs.push(obj);
    return objs.length - 1;
  }

  function channels(format) {
    if (format === GL_RED) return 1;
    if (format === GL_RGB) return 3;
    return 4; // RGBA and friends
  }

  function adaptShaderSource(type, src) {
    let out = src.replace("#version 330 core", "#version 300 es");
    if (type === GL_FRAGMENT_SHADER && !out.includes("precision ")) {
      out = out.replace("#version 300 es", "#version 300 es\nprecision mediump float;");
    }
    return out;
  }

  const wilhelm = {
    // canvas / environment -------------------------------------------------
    js_setup_canvas(width, height) {
      canvas.width = width;
      canvas.height = height;
      // MSAA is a context-creation attribute in WebGL (the native backend's
      // GLFW_SAMPLES=4 hint maps to antialias: true).
      gl = canvas.getContext("webgl2", { antialias: true });
      if (!gl) throw new Error("WebGL2 is not available in this browser");
      gl.viewport(0, 0, width, height);
    },
    js_canvas_width: () => canvas.width,
    js_canvas_height: () => canvas.height,
    js_now: () => performance.now() / 1000,
    js_log: (ptr, len) => console.log(str(ptr, len)),

    // GL state -------------------------------------------------------------
    js_gl_clear_color(r, g, b, a) {
      // Contract semantics: the native shim fuses glClear into this call.
      gl.clearColor(r, g, b, a);
      gl.clear(gl.COLOR_BUFFER_BIT);
    },
    js_gl_viewport: (x, y, w, h) => gl.viewport(x, y, w, h),
    js_gl_get_viewport: (ptr) => i32(ptr, 4).set(gl.getParameter(gl.VIEWPORT)),
    js_gl_enable(cap) {
      if (cap !== GL_MULTISAMPLE) gl.enable(cap); // invalid enum in ES 3.0
    },
    js_gl_blend_func: (s, d) => gl.blendFunc(s, d),

    // shaders / programs ---------------------------------------------------
    js_gl_create_shader(type) {
      const id = alloc(gl.createShader(type));
      shaderTypes.set(id, type);
      return id;
    },
    js_gl_shader_source(shader, ptr, len) {
      gl.shaderSource(objs[shader], adaptShaderSource(shaderTypes.get(shader), str(ptr, len)));
    },
    js_gl_compile_shader(shader) {
      gl.compileShader(objs[shader]);
      if (!gl.getShaderParameter(objs[shader], gl.COMPILE_STATUS)) {
        console.error("shader compile failed:", gl.getShaderInfoLog(objs[shader]));
      }
    },
    js_gl_delete_shader(shader) {
      gl.deleteShader(objs[shader]);
      objs[shader] = null;
    },
    js_gl_get_shaderiv(shader, pname) {
      const v = gl.getShaderParameter(objs[shader], pname);
      return typeof v === "boolean" ? (v ? 1 : 0) : v | 0;
    },
    js_gl_create_program: () => alloc(gl.createProgram()),
    js_gl_attach_shader: (p, s) => gl.attachShader(objs[p], objs[s]),
    js_gl_link_program(program) {
      gl.linkProgram(objs[program]);
      if (!gl.getProgramParameter(objs[program], gl.LINK_STATUS)) {
        console.error("program link failed:", gl.getProgramInfoLog(objs[program]));
      }
    },
    js_gl_delete_program(program) {
      gl.deleteProgram(objs[program]);
      objs[program] = null;
    },
    js_gl_use_program: (p) => gl.useProgram(objs[p]),

    // buffers / vertex arrays ----------------------------------------------
    js_gl_gen_buffer: () => alloc(gl.createBuffer()),
    js_gl_bind_buffer: (target, b) => gl.bindBuffer(target, objs[b]),
    js_gl_buffer_data(target, ptr, size, usage) {
      if (ptr === 0) gl.bufferData(target, size, usage); // allocate-only
      else gl.bufferData(target, bytes().subarray(ptr, ptr + size), usage);
    },
    js_gl_buffer_sub_data(target, offset, ptr, size) {
      gl.bufferSubData(target, offset, bytes().subarray(ptr, ptr + size));
    },
    js_gl_delete_buffer(b) {
      gl.deleteBuffer(objs[b]);
      objs[b] = null;
    },
    js_gl_gen_vertex_array: () => alloc(gl.createVertexArray()),
    js_gl_bind_vertex_array: (v) => gl.bindVertexArray(objs[v]),
    js_gl_delete_vertex_array(v) {
      gl.deleteVertexArray(objs[v]);
      objs[v] = null;
    },
    js_gl_vertex_attrib_pointer: (index, size, type, normalized, stride, offset) =>
      gl.vertexAttribPointer(index, size, type, !!normalized, stride, offset),
    js_gl_enable_vertex_attrib_array: (index) => gl.enableVertexAttribArray(index),
    js_gl_vertex_attrib_divisor: (index, divisor) => gl.vertexAttribDivisor(index, divisor),
    js_gl_vertex_attrib_4f: (index, x, y, z, w) => gl.vertexAttrib4f(index, x, y, z, w),

    // textures ---------------------------------------------------------------
    js_gl_active_texture: (unit) => gl.activeTexture(unit),
    js_gl_gen_texture: () => alloc(gl.createTexture()),
    js_gl_bind_texture: (target, t) => gl.bindTexture(target, objs[t]),
    js_gl_tex_parameteri: (target, pname, param) => gl.texParameteri(target, pname, param),
    js_gl_generate_mipmap: (target) => gl.generateMipmap(target),
    js_gl_pixel_storei: (pname, param) => gl.pixelStorei(pname, param),
    js_gl_delete_texture(t) {
      gl.deleteTexture(objs[t]);
      objs[t] = null;
    },
    js_gl_tex_image_2d(target, level, internalformat, w, h, border, format, type, ptr) {
      const data = ptr === 0 ? null : bytes().subarray(ptr, ptr + w * h * channels(format));
      gl.texImage2D(target, level, internalformat, w, h, border, format, type, data);
    },
    js_gl_tex_sub_image_2d(target, level, xo, yo, w, h, format, type, ptr) {
      gl.texSubImage2D(
        target, level, xo, yo, w, h, format, type,
        bytes().subarray(ptr, ptr + w * h * channels(format))
      );
    },

    // draws ------------------------------------------------------------------
    js_gl_draw_arrays: (mode, first, count) => gl.drawArrays(mode, first, count),
    js_gl_draw_arrays_instanced: (mode, first, count, n) =>
      gl.drawArraysInstanced(mode, first, count, n),
    js_gl_draw_elements: (mode, count, type, offset) => gl.drawElements(mode, count, type, offset),

    // uniforms ---------------------------------------------------------------
    js_gl_get_uniform_location(program, ptr, len) {
      const name = str(ptr, len);
      const key = program + ":" + name;
      let idx = uniformCache.get(key);
      if (idx === undefined) {
        const loc = gl.getUniformLocation(objs[program], name);
        if (loc === null) {
          idx = -1;
        } else {
          uniforms.push(loc);
          idx = uniforms.length - 1;
        }
        uniformCache.set(key, idx);
      }
      return idx;
    },
    js_gl_uniform_1f: (l, x) => gl.uniform1f(uniforms[l], x),
    js_gl_uniform_2f: (l, x, y) => gl.uniform2f(uniforms[l], x, y),
    js_gl_uniform_3f: (l, x, y, z) => gl.uniform3f(uniforms[l], x, y, z),
    js_gl_uniform_4f: (l, x, y, z, w) => gl.uniform4f(uniforms[l], x, y, z, w),
    js_gl_uniform_matrix_4fv: (l, count, transpose, ptr) =>
      gl.uniformMatrix4fv(uniforms[l], !!transpose, f32(ptr, 16 * count)),
  };

  fetch("shapes_wasm.wasm")
    .then((r) => {
      if (!r.ok) throw new Error("failed to fetch shapes_wasm.wasm (" + r.status + ")");
      return r.arrayBuffer();
    })
    .then((buf) => WebAssembly.instantiate(buf, { wilhelm }))
    .then(({ instance }) => {
      memory = instance.exports.memory;
      instance.exports.wasm_init();

      window.addEventListener("resize", () => {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        instance.exports.wilhelm_dispatch_resize(canvas.width, canvas.height);
      });

      const loop = () => {
        instance.exports.wasm_frame();
        requestAnimationFrame(loop);
      };
      requestAnimationFrame(loop);
    })
    .catch((e) => {
      console.error(e);
      const pre = document.createElement("pre");
      pre.textContent = String(e);
      document.body.appendChild(pre);
    });
})();
