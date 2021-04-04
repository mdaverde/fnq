# fnq (pronounced FUNK)

A flock-based approach to queuing Unix tasks & processes

## How to use

Set `FNQ_DIR` in your env to dictate where to store queue files. Defaults to `$(pwd)`

```shell
$ FNQ_DIR=/tmp/fnq fnq [--quiet | --clean] cmd
```

### Example

```shell
$ fnq task1 # Looks in PATH for task1
fnq1617220638670.52957
$ fnq task2 # Queues future tasks
fnq1617221011799.53621
$ fnq task3
fnq1617221184552.54371
$ ls $FNQ_DIR
fnq1617220638670.52957  fnq1617221011799.53621  fnq1617221184552.54371
$ fnq --tap fnq1617221011799.53621 # Will check if task is running
$ fnq --wait # Will block until last task finishes
```

### Flags

#### `--quiet`

No stdout

#### `--clean`

Deletes queue file in `$FNQ_DIR` after task completes

#### `--wait [fnq_queue_file.pid]`

Accepts a queue output file to wait for, otherwise blocks and waits for entire queue to finish

#### `--tap [fnq_queue_file.pid]`

Accepts a queue output file to determine if running, otherwise determines success based if entire queue if finished


