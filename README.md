# dm-playground_controller

Basic rust controller application communicating over serial for [dm-playground](https://github.com/spacestation13/dm-playground).

Protocol is uni-directional in the way commands are issued. The client always issues command and the server responds
with either:
- `OK`
- Data followed by `\nOK`
- `ERR <base64 error>\n`

Strings and binary data must always be base64 encoded.

For commands, spaces should be used to seperate arguments. 

For responses, line breaks should be used as seperator when possible for response data

## Commands
### Poll
#### Command 
`poll`
#### Response

```
(<type> <data>\n)*OK
```
type: (STR) Type of the poll event

data: (ANY) Data associated with the poll event
#### Example
```
> poll
< stdout 12345 Q29tcGlsaW5nLi4uCkNvbXBpbGVkIQ==
< pidexit 12345 0
< unzip L3RtcC9kbS1wbGF5Z3JvdW5kL2J5b25kLTUxMy4xMjQ1Lw==
< OK
```

___

### Unzip
#### Command
`unzip <inPath>`

path: (B64_STR) Absolute path to a .zip file

The file should be extracted to a subdirectory in /tmp/dm-playground. An event will be emitted once the extraction is complete
#### Response
```
<outPath>
OK
```
outPath: (B64_STR) Absolute path to a directory that will contain the extracted contents of the zip file
#### Example
```
> unzip L21udC9ob3N0L2J5b25kL2J5b25kLTUxMy4xMjQ1LnppcA==
< L3RtcC9kbS1wbGF5Z3JvdW5kL2J5b25kLTUxMy4xMjQ1Lw==
< OK
```

#### Poll Events
##### Unzipped
###### Format
unzipped: `unzipped <outPath>`

outPath: (B64_STR) Absolute path to a directory that contains the extracted contents of the zip file
###### Example
Ex: `unzipped L3RtcC9kbS1wbGF5Z3JvdW5kL2J5b25kLTUxMy4xMjQ1Lw==`

___

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
___
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

exitCode: (NUM) exit code of the subprocess that exit

###### Example
`pidexit 12345 127`
___
##### Stdout
###### Format
`stdout <pid> <chunk>`

pid: (NUM) Process ID of the subprocess that output on stdout

chunk: (B64) Base64 encoded data that was output

###### Example
`stdout 12345 Q29tcGlsbGluZy4uLi5cbkNvbXBpbGVkIQ==`
___
##### Stderr
###### Format
`stderr <pid> <chunk>`

pid: (NUM) Process ID of the subprocess that output on stderr

chunk: (B64) Base64 encoded data that was output

###### Example
`stdout 12345 V2FybmluZzogYnJ1aCBtb21lbnQ=`