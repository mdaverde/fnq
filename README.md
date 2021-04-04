# fnq (pronounced FUNK)

A flock-based approach to queuing Unix processes & tasks

## How to use

```shell
$ fnq [-quiet/-q | --cleanup] cmd
```

### Example

```shell
$ fnq job1 # Looks in path for job1
$ fnq job2
$ fnq job3
$ fnq --tap fnq1617221011732.53617 # Will check if job is running
$ fnq --wait # Will block until last job finishes
```

### Flags

#### `--quiet / -q`

#### `--clean / -c`

#### `--wait / -w`

Accepts a fnq output file to wait for 

#### `--tap / -t`

Accepts a fnq output file to tap process status

#### `--force-kill-all`

Convenience flag to clear rest of queued ops. If you want to kill a currently running process, you can run pkill on the process id. 

### How it works

## Guarantees

### Tested on

## Maintainer