# Logan

A simple CLI tool to make working with log files easier.

## Usage

Logan supports multiple "processors" which determines the output format. It can be used to print out lines with
different colors, define "events" to capture lines between a "start" and an "end" or simply filter the output (grep).
Logan can be used with CLI commands or a config file. Using it with CLI commands are simpler but limited to a single
processor. The following processors are available right now:

### Coloring lines

You can print out lines with different colors with the __colorize__ command:

```
$ logan example.log colorize -p "INFO" 28 -p "WARN" 24 -p "ERROR" 88
```

Where the first parameter is a regex pattern and the second is a color. You can also define a prefix that will be
prepended to every pattern with the _-P_ or _--prefix_ argument:

```
$ logan example.log colorize -P "[\d]{4}-[\d]{2}-[\d]{2} [\d]{2}:[\d]{2}:[\d]{2} " -p "INFO" 28 -p "WARN" 24 -p "ERROR" 88
```

#### Note

The color parameter refers to a palette index of your terminal app. It will be possible to define RGB colors in a
future release. You have to check out your terminal's color palette to know which index is which color.

### Events

You can define events with the __events__ command. It takes a start and end pattern that will define an event.
E.g. if you want to check out mouse clicks in a log file you can use the following:

```
$ logan example.log events -P "[\d]{4}-[\d]{2}-[\d]{2} [\d]{2}:[\d]{2}:[\d]{2} INFO " -c 28 "Mouse left down" "Mouse left up"
```

Which will print all lines between the occurence (including both ends) of lines containing "Mouse left down" and
"Mouse left up".

### States

You can define a pattern that will count as a state change in your application with the __states__ argument. It will
print out only lines that match the pattern. It will also print out the last line where the state changed.

```
$ logal example.log states -P "[\d]{4}-[\d]{2}-[\d]{2} [\d]{2}:[\d]{2}:[\d]{2} " -c 28 "INFO Set state to"
```

On itself it's basically just a grep but you can combine these features by using a config file.

### Using a config file

You can use a config file to combine these log processors. An example config file that combines all the previous commands
would look like this:

```json
{
    "prefix": "[\\d]{4}-[\\d]{2}-[\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2} ",
    "pattern_colors": [
        { "pattern": "INFO", "color": "28" },
        { "pattern": "WARN", "color": "24" },
        { "pattern": "ERROR", "color": "88" }
    ],
    "event_patterns": [
        {
            "start_pattern": "INFO Mouse left down",
            "end_pattern": "INFO Mouse left up",
            "color": "29"
        }
    ],
    "state_patterns": [
        { "pattern": "INFO Set state to", "color": "30" }
    ]
}
```

and you can use this config file with the following command line parameters:

```
$ logan example.log use-config example.json
```

With a config file you can define multiple event and state as well.

## Installation

Currently you have to compile Logan to run it. The plan is to support all three major platform (Windows, Linux, MacOS)
with pre-built binaries. You can also use __*cargo run -- ARGS*__ to run Logan.
