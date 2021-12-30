# dm-playground_controller

Basic rust controller application communicating over serial for [dm-playground](https://github.com/spacestation13/dm-playground).

Protocol is uni-directional in the way commands are issued. The client always issues command and the server responds
with either:

- `HELLO\0` (On startup)
- `<data>\nOK\0`
- `<base64 error>\nERR\0`

Strings and binary data must always be base64 encoded.

Only UTF-8 is supported.

For commands, spaces should be used to seperate arguments.

For responses, line breaks should be used as seperator when possible for response data

## Commands

### Poll

#### Command

`poll`

#### Response

```
(<typ> <data>\n)*OK
```

typ: (STR) Type of the poll event

data: (ANY) Data associated with the poll event

#### Example

```
> poll
< stdout 12345 Q29tcGlsaW5nLi4uCkNvbXBpbGVkIQ==
< pidexit 12345 0
< unzip L3RtcC9kbS1wbGF5Z3JvdW5kL2J5b25kLTUxMy4xMjQ1Lw==
< OK
```

---

### Signal

#### Command

`signal <pid> <signal>`

pid: (INT) PID of the process to signal

signal: (INT) signal to send

#### Response

`OK`

#### Example

```
> signal 12345 7
< OK
```

---

### Run

#### Command

`run <path> <args> <env>`

path: (B64_STR) Path of the executable

args: (B64_STR) Arguments to run the executable with

env: (B64_STR) Environment variables to pass to the executable in the following format: `VAR1=VAL1;VAR2=VAL2;`. Take care to escape `\;`, `\=` and `\\` properly.

#### Response

```
<pid>
OK
```

pid: (INT) process ID of the created process

#### Example

```
> run L3RtcC9kbS1wbGF5Z3JvdW5kL2J5b25kLTUxMy4xMjQ1L2Jpbi9EcmVhbU1ha2Vy L21udC9ob3N0L2NvZGUvMjUuZG1l UEFUSD0vYmluO0hUVFBfUFJPWFk9bG9jYWxob3N0Ojg4ODg7REVWPTE=
< 12345
< OK
```

#### Poll Events

##### Exit

###### Format

`pidexit <pid> <exitCode>`

pid: (NUM) Process ID of the subprocess that exit

exitCode: (NUM) exit code of the subprocess that exit.
0-255: Exited via exit(), exit code is exitCode
256: Exit code or signal could not be determined
257-inf: Exited by being kill()'d. Signal is (exitCode - 256)

###### Example

`pidexit 12345 127`

---

##### Stdout

###### Format

`stdout <pid> <chunk>`

pid: (NUM) Process ID of the subprocess that output on stdout

chunk: (B64) Base64 encoded data that was output

###### Example

`stdout 12345 Q29tcGlsbGluZy4uLi5cbkNvbXBpbGVkIQ==`

---

##### Stderr

###### Format

`stderr <pid> <chunk>`

pid: (NUM) Process ID of the subprocess that output on stderr

chunk: (B64) Base64 encoded data that was output

###### Example

`stdout 12345 V2FybmluZzogYnJ1aCBtb21lbnQ=`
