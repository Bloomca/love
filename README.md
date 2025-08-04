## Love

This is a simple TUI code editor application. It is under heavy development, although it mostly works as a text editor already. You can scroll, type, delete, copy, paste and select text. Some features I am working on right now:

- support undo/redo
- add code highlighting via [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- add command bar

This editor is a modal editor, but the insert mode is the default one, and most operations should be available from it.

![Screenshot](./screenshot.jpg)

## Running

You can run it locally (to quit, press `CTRL + Q`) by specifying any file:

```
cargo run Cargo.lock
```