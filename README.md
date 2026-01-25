# Supera
## Supervisor for Running code Asynchronously

# Command
To use Supera a 'message' must be defined, an instance of the message must know
how to do work. That is achieved by implementing the trait `Command`. The
message is then used as a command for an async runner.

All commands must define a signal to halt execution of the runner.
To acheive that `StopRunner` or `SimpleStop` can be defined.

`StopRunner` should be used in cases where there's valuable information to be
passed down to the runner when they are to be halted. Otherwise `SimpleStop` should be used.

# Runner
An async runner can receive a commands and execute them. They are entirely
implementation-specific and created solely for the Managers.

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

[^Linked]:
    A Linked manager will create a single-use channel for _each request_ sent.
    This incurs some cost but greatly simplifies their usage.
