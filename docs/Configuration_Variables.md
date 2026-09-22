# Configuration Variables

The storage supports different types of configuration variables. 

The configuration variables are stored in a ConfigStore struct, which is a singleton that can be accessed from anywhere in the code. 

The ConfigStore struct has a get and set function that allows you to get and set the value of a configuration variable.

## HOW TO ADD A NEW CONFIGURATION VARIABLE:
- Add a new static variable with name and value in config/registry.rs
- Add the reference in the CONFIG_VARIABLES registry in the same file

## HOW TO ADD A NEW VALUE TYPE:
- Add a new type in ConfigStorage of config/mod.rs
- Add new row in get and set of config storage how would you encode and decode the value of the new type (mostly will be the same without any changes only special case like float will need to be converted to bits and stored in AtomicU32)
- Add a new type in ConfigValue, this can helps return the value from getter and setter of the ConfigStore

## Notes:
- Using set_config in telecommand must specify the correct type for the variable being set. The type of the variable will be determined in the get and set function of the ConfigStore implementation
- ConfigStore will be storing only Atomic types (bool, u32, u8, ...). If you need to store a type that is not available in Atomic types, you can convert it to bits and store in AtomicU32 or AtomicU64. Or you could use Mutex for more complex types.
    - For example, if you want to store a f32, you can convert it to u32 using 21.3_f32.to_bits() (21.3 is the example float you wanna store here). You might have to do extra stuff to convert in get and set functions to convert back and forth.
- String type is not yet thought about and tested but it could be possible with Mutex
