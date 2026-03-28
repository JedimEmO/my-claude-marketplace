import rust from "@wasm-tool/rollup-plugin-rust";
import dev from "rollup-plugin-dev";
import livereload from "rollup-plugin-livereload";
import terser from "@rollup/plugin-terser";
import copy from "rollup-plugin-copy";

const is_watch = !!process.env.ROLLUP_WATCH;
const is_release = !!process.env.RELEASE;

export default {
  input: {
    app: "../crates/app-frontend/Cargo.toml",
  },
  output: {
    dir: "dist/js",
    format: "es",
    sourcemap: true,
  },
  plugins: [
    rust({
      optimize: { release: is_release },
      verbose: true,
      extraArgs: {
        cargo: [
          ...(!is_release ? ["--config", "profile.dev.debug=true"] : []),
        ],
        wasmBindgen: !is_release ? ["--debug", "--keep-debug"] : [],
        wasmOpt: [
          "-Oz",
          "--enable-bulk-memory-opt",
          "--enable-nontrapping-float-to-int",
        ],
      },
    }),
    copy({
      targets: [
        { src: "index.html", dest: "dist/" },
      ],
    }),
    is_watch &&
      dev({
        dirs: ["dist"],
        port: 8080,
        silent: true,
        spa: true,
        proxy: [
          { from: "/api/*", to: "http://localhost:3000" },
        ],
      }),

    is_watch && livereload("dist"),

    !is_watch && terser(),
  ],
};
