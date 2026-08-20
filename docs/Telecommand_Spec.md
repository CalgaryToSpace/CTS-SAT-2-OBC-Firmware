# Telecommand Specification
The doc explains how the telecommand are structured.

## Examples
```
hello_world()
set_config(config_demo_variable1, u32(123))
get_config(config_demo_variable1)@tsexec=1787188892818
```

## Format

1. Command name: e.g., `hello_world`
2. Arguments: e.g., `(arg1, arg2, ...)`
    + Argument are separated by commas and positional
    + Arguments are required and no excess arguments are allowed
3. Suffix 1 (tsexec): e.g., `@tsexec=1787188892818`
    + In milliseconds since unix epoch
    + Suffix is used to specify the timestamp of the command execution
    + Suffix is optional (default to 0 which executes the command immediately)
4. Right now the telecommand must ends in `\r\n` and might be changed in the future with a special character.

## Warning

These characters cannot be used in the telecommand:

```
( ) @ = ,
```
