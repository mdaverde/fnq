<div align="center">
	<h1>fnq (pronounced FUNK)</h1>
	<p>
		<b>A flock-based approach to queuing Unix tasks & processes</b>
	</p>
	<br>
</div>

## Usage

Set `FNQ_DIR` in your env to dictate where to store queue files. Defaults to `$(pwd)`

```shell
$ FNQ_DIR=/tmp/fnq fnq [--quiet | --clean] cmd
```

Protip: since `fnq` uses `FNQ_DIR` to determine queue state, you can create an entirely new queue by changing `FNQ_DIR`

### Example

```shell
$ fnq task1 # Looks in PATH for task1
fnq1617220638670.52957
$ fnq task2 taskarg1 taskarg2 # Queues future tasks
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

Note that the std{out,error} from the task cmd will still be saved to the corresponding queue file

#### `--clean`

Deletes queue file in `$FNQ_DIR` after task completes

#### `--wait [fnq_queue_file.pid]`

Accepts a queue output file to wait for, otherwise blocks and waits for entire queue to finish

#### `--tap [fnq_queue_file.pid]`

Accepts a queue output file to determine if running, otherwise determines success based if entire queue if finished

## Install

### Build from source

After git cloning this repo, you can install as a cargo crate through

```shell
$ cargo install --path path_to_repo
```

This should make `fnq` available everywhere assuming your cargo crates are in `$PATH`

## About

This package depends on [nix](https://github.com/nix-rust/nix) to abstract over Unix flock operations, so presumably if nix works on a platform, this bin should work.


## License

MIT

Maintained by [Milan](https://mdaverde.com)

