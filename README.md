# fnq (pronounced FUNK)

A flock-based approach to queuing Unix processes & tasks

## How to use

```shell
$ fnq [-quiet/-q | --cleanup] cmd
```

### Example

```shell
$ fnq task1 # Looks in PATH for task1
$ fnq task2 # Queues future tasks
$ fnq task3
$ fnq --tap fnq1617221011732.53617 # Will check if task is running
$ fnq --wait # Will block until last task finishes
```

### Flags

#### `--quiet / -q`

#### `--clean / -c`

#### `--wait / -w`

Accepts a fnq output file to wait for 

#### `--tap / -t`

Accepts a fnq output file to tap process status

## How it works

