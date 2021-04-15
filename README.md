# string_art

> Transform an image into string art

## Usage

```
USAGE:
    string_art [FLAGS] [OPTIONS] --input-filepath <FILEPATH>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Output debugging messages.

OPTIONS:
    -l, --auto-color-limit <INTEGER>       Limit the number of colors chosen when using --style=auto-color [default: 2]
    -d, --data-filepath <FILEPATH>         The script will write operation information as a JSON file if this filepath
                                           is given. The operation information includes argument values, starting and
                                           ending image scores, pin locations, and a list of line segments between pins
                                           that form the final image.
    -g, --gif-filepath <FILEPATH>          Location to save a gif of the creation process
    -x, --hex-color <HEX_COLOR>...         An RGB color in hex format `#RRGGBB` specifying the color of a string to use.
                                           Can be specified multiple times to specify multiple colors of strings.
                                           Only used when --style is one of 'color-on-black' or 'color-on-white'.
    -i, --input-filepath <FILEPATH>        Path to the image that will be rendered with strings.
    -m, --max-strings <INTEGER>            The maximum number of strings in the finished work. [default: 4294967295]
    -o, --output-filepath <FILEPATH>       Location to save generated string image.
    -r, --pin-arrangement <ARRANGEMENT>    Should the pins be arranged on the image's perimeter, or in a grid across the
                                           entire image, or in the largest possible centered circle, or scattered
                                           randomly? [default: perimeter]  [possible values: perimeter, grid, circle,
                                           random]
    -c, --pin-count <INTEGER>              How many pins should be used in creating the image (approximately). [default:
                                           200]
    -p, --pins-filepath <FILEPATH>         Location to save image of pin locations.
    -s, --step-size <FLOAT>                Used when calculating a string's antialiasing. Smaller values -> finer
                                           antialiasing. [default: 1]
    -a, --string-alpha <FLOAT>             How opaque or thin each string is. `1` is entirely opaque, `0` is invisible.
                                           [default: 1]
    -t, --style <STYLE>                    The style of image to create.

                                               [White|Black|Color] threads on [White|Black] background.
                                               Automatically pick all colors.

                                           When using color threads, pass the desired colors with the --hex-color
                                           argument.
                                           When using 'auto-color', the program will try to detect the best colors
                                           (background & strings) for approximating the image. Limit the number of
                                           string colors chosen with --auto-color-limit. [default: white-on-black]
                                           [possible values: white-on-black, black-on-white, color-on-black, color-on-
                                           white, auto-color]
```
