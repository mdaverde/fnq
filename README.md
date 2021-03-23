# fnq (pronounced FUNK)

A flock-based approach to queuing Unix processes & tasks

## How to use

```shell
$ fnq [-quiet/-q | --cleanup] cmd
```

### Flags

#### `--quiet / -q`

#### `--clean / -c`

#### `--watch / -w`

Accepts a fnq output file to wait for 

#### `--test / -t`

Accepts a fnq output file to wait for

#### `--force-kill-all`

Convenience flag to clear rest of queued ops. If you want to kill a currently running process, you can run pkill on the process id. 

### How it works

## Guarantees

### Tested on

## Maintainer