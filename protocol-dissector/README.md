# protocol-dissector
Wireshark dissector for SOE's Reliable UDP protocol version 3 (version string `CGAPI_527`). 
Modified version of `https://github.com/misterupkeep/soe-dissector`.

## Dependencies
The Lua dissector depends on `LibDeflate` for decompressing packets.
You can acquire it from `https://github.com/SafeteeWoW/LibDeflate` or -
Check the Lua version on your Wireshark; you can use `luarocks` to
install it:
```sh
luarocks-5.2 install LibDeflate
```

## Installation
Place or symlink the `.lua` file onto Wireshark's Lua plugin path. You
can find out all the directories Wireshark scans in `Help > About >
Folders`, or run:
```sh
mklink "C:\Users\path\to\your\Wireshark\plugins\3.4\soe-dissector.lua" "C:\path\to\your\project\lua\soe-dissector.lua"
```
