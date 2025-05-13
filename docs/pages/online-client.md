# Online Client

### [<-](../README.md)

!WARNING!
These docs are outdated and work in progress. Right now you have to clone the repo and run the server directly, then run the online client directly.

## Overview

The Hygg online client allows you to read text files with progress tracking that is synchronized with a server. This means you can start reading on one device and continue from the same position on another device.

## Features

- **Progress Tracking**: Your reading position is saved and can be resumed across different devices
- **File Synchronization**: Upload local files to the server for access anywhere
- **Multi-User Support**: Multiple people can read the same file simultaneously
- **Lock Management**: Only one user can modify progress at a time, others can read in view-only mode

## Usage

### Reading a File

To read a file from the server:

```sh
hygg read sample.txt --user user1
```

or simply:

```sh
hygg sample.txt --user user1
```

### Uploading a File

To upload a local file to the server for syncing:

```sh
hygg upload /path/to/local/file.txt --user user1
```

This will make the file available for reading through the online client.

## Command Line Options

- `--user`, `-u`: Set the user ID (defaults to a random UUID if not specified)
- `--col`, `-c`: Set the column width for text display (defaults to 110)
- `--help`, `-h`: Display help information

## Lock Management

When a user opens a file, they attempt to acquire a lock for that file. If the file is already locked by another user:

1. The file will open in read-only mode
2. A READ-ONLY indicator will appear in the interface
3. Reading progress will not be updated

When a user closes the file, the lock is automatically released, allowing other users to acquire the lock and update progress.
