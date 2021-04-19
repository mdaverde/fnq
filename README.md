<div align="center">
	<h1>fnq (pronounced FUNK)</h1>
	<p>
		A flock-based approach to queuing Unix tasks & processes
	</p>
	<br>
</div>

## Usage

Set `FNQ_DIR` in your env to dictate where to store queue files. Defaults to `$(pwd)`

```shell
$ fnq [--quiet | --clean] cmd
```

Protip: since `fnq` uses `FNQ_DIR` to determine queue state, you can create an entirely new queue by changing `FNQ_DIR`

### Example

```shell
$ fnq ./task1 # Can also look in PATH
fnq1617220638670.52957
$ fnq ./task2 taskarg1 taskarg2 # Queues future tasks
fnq1617221011799.53621
$ fnq -q ./task3
$ ls $FNQ_DIR
fnq1617220638670.52957  fnq1617221011799.53621  fnq1617221184552.54371
$ fnq --tap fnq1617221011799.53621 # Will check if task is running
$ fnq --block # Will block until last task finishes
```

### Flags

#### `--quiet / -q`

No stdout

**Note:** std{out,error} from the task cmd will still be saved to the corresponding queue file

#### `--clean / -c`

Deletes queue file in `$FNQ_DIR` after task completes

#### `--block / -b <queuefile.pid>`

Accepts a queue output file to wait for, otherwise waits/blocks for entire queue to finish

#### `--tap / -t <queuefile.pid>`

Accepts a queue output file to determine if running, otherwise determines success based if entire queue if finished

#### `--watch / -w <queuefile.pid>`

Similar to `--block` but will print to stdout contents of the currently running queue files

## Install

### Cargo

If you're using a recent version of Cargo, you can see the `cargo install` command:

```shell
$ cargo install fnq
```

### Build from source

After git cloning this repo, you can install as a cargo crate through

```shell
$ cargo install --path path_to_repo
```

This should make `fnq` available everywhere assuming your cargo crates are in `$PATH`

## About

Much of the functionality here is heavily inspired by [nq](https://github.com/leahneukirchen/nq) (written in C).

This package depends on [nix](https://github.com/nix-rust/nix) to abstract over Unix flock operations, so presumably if nix works on a platform, this bin should work. Part of the work needed here is creating a suitable testing suite to run across different machines

Currently tested on linux x86_64 during development

## License

MIT

Maintained by [Milan](https://mdaverde.com)

