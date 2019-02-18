# crafting-helper

A small generic tool to calculate crafting times and resource usage.

Currently there are only template recipes in the data file. To make this tool usable you have to add your own. See [Data Format](#data-format)

## Command Line Usage
```
Usage:
  crafting.exe [OPTIONS] SEARCH STRINGS [...]

Crafting helper.

Positional arguments:
  search strings        the name or id of the part you want to build. [partial
                        names are supported (uses the first matching part),
                        whitespaces are allowed and don't have to be escaped]

Optional arguments:
  -h,--help             Show this help message and exit
  -p,--path PATH        the path to the data file.
  -d,--details          print details.
  -a,--amount AMOUNT    amount needed.
  -D,--descending       Sort descending.
  -s,--search           search for all matching parts. prints info for part if
                        only one is found.
  -c,--count            show item count in data file
  -l,--list             list all items in data file
```

## Data Format
```toml
[basic-construct]        # the id of the item
name = "Basic Construct" # the full name of the item (used for sorting, if tier is the same)
tier = 5                 # the tier of the item (used for sorting)
time = { seconds = 8 }   # the time needed to craft this item. Excluding the time for the sub-parts
                         # time supports the fields: weeks, days, hours, minutes, seconds.
                         # e.g.: { hours = 3, minute = 2, seconds = 10 }
variations = [           # all variations of the item that can be crafted with the same materials
    "red",
    "blue",
    "green"
]
```