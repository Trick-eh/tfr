# TFR

tfr or trick's fast reader (i know, original isn't it) is an application/utility 
that makes use of techniques such as rapid serial visual presentation and optimal
recognition point to enhance the user's ability to read fast.

## Installation


clone the repo and then run:

```bash

  $ cargo build --release
```

then, in the `target/release` directory you'll find the respective binary/executable.

Disclaimer: Windows may warn against the execution of the build command or the binary
because it will autocreate a file in its own directory inside `AppData/Roaming/` in 
order to persist the latest theme, wpm ratio, file path and word number.

## Supported File Types

Up to this moment, tfr provides support for `.md`, `.txt`, `.docx`, `.pdf`, `.epub`.
Have in mind tho that both `pdf` and `docx` won't necesarily render properly because
of certain decoratons and unicode characters.
