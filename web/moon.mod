// Learn more about moon.mod configuration:
// https://docs.moonbitlang.com/en/latest/toolchain/moon/module.html
//
// To add a dependency, run this command in your terminal:
//   moon add moonbitlang/x
//
// Or manually declare it in `import`, for example:
// import {
//   "moonbitlang/x@0.4.6",
// }

name = "aidlc-handson/web"

version = "0.1.0"

readme = "README.mbt.md"

repository = ""

license = "Apache-2.0"

keywords = [ ]

preferred_target = "js"

description = "研修コース e チケット購入サイトの画面（MoonBit・TEA。rabbita と mizchi/js_web）"

import {
  "moonbit-community/rabbita@0.17.0",
  "mizchi/js_web@0.13.1",
  "mizchi/js_core@0.13.0",
}
