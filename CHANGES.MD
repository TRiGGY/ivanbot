# 0.1.0

First release.

# 0.2.0

Added channel [lock,unlock]. Lock enables the bot to only listen to commands in the current channel where the command is
executed.

# 0.3.0

Changed configuration format to JSON breaking all previous config files. (They will be auto-recreated)

# 0.4.0

Fixed commands that didn't parse 64 bit steam ID's correctly. Added -skin feature that allows you to assign a skin to
everyone.

# 0.5.0

Added delay between skin changes to prevent crashes.

# 0.6.0

Added support for WW2 update. New basic commands (Kill, MapList, SetCash)
Support to change map to default maps New setting: GunMode {WW2, Modern, Random} (default Modern). Will let map votes
from maps with game mode be created with either WW2GUN instead or generate them randomly between GUN and WW2GUN, Modern
is normal behavior.

# 0.7.0
Added Koth support. 
Added better help messages to commands.
New commands added:
-team balance
-team shuffle
Shuffle will generate new random teams.
Balance will move as few people as possible from 1 team to the other in order to balance out team differences. Making the difference <= 1.

#0.7.1
Added OITC support.
