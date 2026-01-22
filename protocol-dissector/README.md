# protocol-dissector
Wireshark dissector for SOE's Reliable UDP protocol version 3 (version string `CGAPI_527`).
Modified version of `https://github.com/misterupkeep/soe-dissector`.

## Dependencies
The Lua dissector depends on `LibDeflate` for decompressing packets.
You can acquire it from `https://github.com/SafeteeWoW/LibDeflate`.

## Installation
Place the `soerudp.lua` and `LibDeflate.lua` files onto Wireshark's Lua plugin path.
You can find out all the directories Wireshark scans in `Help > About > Folders`.
The default directory on Windows is typically `C:\Users\YourUser\AppData\Roaming\Wireshark\plugins\`.
