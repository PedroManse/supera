# Supera
## Supervisor for Running code Asynchronously[^worker-threads]

# Command
To use Supera a 'message' must be defined, an instance of the message must know
how to be executed. That is achieved by implementing the trait `Command`. The
message is then sent to a worker thread, what the worker does is implementation
specific.

All commands must define a signal to halt execution of the runner.
To acheive that `StopRunner` or `SimpleStop` can be defined.

`StopRunner` should be used in cases where there's valuable information to be
passed down to the runner when they are to be halted. Otherwise `SimpleStop` should be used.

# Runner
Runners are worker threads created for the Manager. What they do is
implementation specific. But the built-in runners acquire messages, execute
them and in some form return the results to the user.

# Manager
An execution manager implements the `CommandRunner` trait. So it's able to:
* Create it self and it's `Runners`
* Have a message sent to a `Runner`
* Close it self and all it's `Runners`

# Native managers
There are four execution managers

Response method | Many runners     | Single runner    |
----------------|------------------|------------------|
Ordered         | `PoolQueueAPI`   | `SingleQueueAPI` |
Linked[^Linked] | `OneShotPoolAPI` | `OneShotAPI`     |

[^worker-threads]:
    Code will *not* execute on an async runtime. It will simply execute on one
    or more worker threads

[^Linked]:
    A Linked manager will create a single-use channel for _each request_ sent.
    This incurs some cost but greatly simplifies their usage.
