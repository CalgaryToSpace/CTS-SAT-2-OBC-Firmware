# get_timestamp(&str) Documentation

## Overview
When typed in the Serialtest menu, it executes `timestamp_ms(timea_input: &str)` in `timekeeping.rs`

It takes a line from a TIMEA log (published by the GNSS built onto the Cube-SAT), extracts all the tokens from it, and converts it to a UNIX timestamp in milliseconds

## Usage
To use get_timestamp() in SerialTest, enter a valid TIMEA log as an argument. Remember to change the time fields to fit your needs (i.e. 2026,10,27,18,55,15545 for 2026, October 27, 6:55:15.545 pm))
get_timestamp(timea_log) -> approx. 1.8 trillion milliseconds depending on time


## Input format
Here is an example of a valid input:
  
  `#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,9,29,19,03,45000,VALID*1100ad64`

This TIMEA log is equivalent to the time 7:03:45.000 pm, September 29, 2026. When inputted into the telecommand, it returns `1790708625000` milliseconds
The important tokens are the ones starting from 2026, going up to "VALID". These are all the important time fields, like year, month, day, hour, minute, and millisecond

## Error handling
The function returns a Result<u64, TimestampError>. TimestampError is a struct designed to handle all potential problems this function could run into:
```
pub enum TimestampError {
    InvalidUTCStatus,
    InvalidTIMEALength,
    ParseIntError,
    InitFailed,
    ValueOutOfRange,
}
```

### InvalidUTCStatus
timestamp_ms() extracts these time fields from the TIMEA log and uses them to calculate the number of milliseconds since UNIX Epoch (Jan 1, 1970). The "VALID" field is important
as it must be VALID for it to work. If it is INVALID, it will return this error

### InvalidTIMEALength
The log must have exactly 22 tokens. Any more or any less, and the function will throw an error. The function parses the input from ',' ';' and '*'. 
If there is a typo or glitch in the input (i.e. a missing ',' or accidental doubles, like ',,'), it will misread it, getting a number of tokens other than 22
A different number means a faulty log, so we cannot parse it and get timestamp from it

### ParseIntError
If the log has the expected 22 tokens, the function will then safely parse the numbers from the time fields. It throws `ParseIntError` if it runs into any problems
while trying to parse numbers. This might be caused by accidental letters mixed with the numbers in their respective fields

### InitFailed
This only occurs if the init() function in `timekeeping.rs` fails to successfully run. A faulty input log will not trigger this error

### ValueOutOfRange
Throws an error if the numbers given in certain fields are beyond the expected bounds. For example, if the month field has 13 or higher, or the hour field has 24 or 25 or higher

Month: minimum 1, maximum 12
Day: minimum 1, maximum 30 or 31 (or 28 or 29 in February)
Hour: minimum 0, maximum 23
Minute: minimum 0, maximum 59
Millisecond: minimum 0, maximum 60999
