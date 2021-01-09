# Introduction
IvanBot is a discord bot that allows control of a Pavlov game server through text communication. All Pavlov RCON commands are implemented and checked for correctness before being send to the Pavlov server. Features that I wanted weren't available in other bots so I decided to try to learn some Rust in the progress of making this bot.
# Features
* All RCON pavlov commands implemented
* Manage map aliases allowing you to -map add *steamworkshop name* for example -alias add https://steamcommunity.com/sharedfiles/filedetails/?id=1454448750 manor_ttt
* Enhanced version of certain commands. For example, -SwitchMap *workshop_url* *gamemode* is able to resolve the map ID automatically.
* Permission system
    * Admin: All commands
    * Moderator: { switchmap | kill | maplist | setcash |rotatemap | alias | map | switchteam | giveitem | givecash | resetsnd | setplayerskin | setlimitedammotype } and User commands
    * User (when ALLOW_USERS=true) { inspectplayer | serverinfo | refreshlist | bothelp }
* Map voting from a pre-configured pool (-map vote start/map vote finish,or wait 30 sec for the vote to end)
* Bot manage (non RCON) commands
    * **admin [add,remove] discord_id_64**          #Add/remove admin users
    * **mod [add,remove] discord_id_64**            #Add/remove moderator users
    * **alias [add,remove] {url/map} alias**        #Create a map alias
    * **alias list**                                #Show all aliases
    * **bothelp**                                   #Help command
    * **mod [add,remove] discord_id_64**            #Add moderator
    * **map add {url/map} gamemode alias**          #Add map to pool
    * **map vote start (X)** #Start map vote with X (optional) choices, default 3
    * **map vote stop** #Conclude the map vote and switch map
    * **map list**
    * **gunmode {WW2, Modern, Random}** #When maps are entered with regular gamemode GUN. WW2 will change map votes to be WW2GUN, Modern is the default and the normal GUN behavior. Random chooses either. Note that SwitchMap is not affected, only voting.
    * **skin {random, clown, prisoner, naked, farmer, russian, nato, german, soviet, us}** #Change all current players to either a random skin or a specific skin
    * **skin shuffle {true/false}** #When enabled will execute "skin random" 90 seconds after a vote is completed
  
# Installation 
First you'll need to gather some things:
1. The IP+port ("127.0.0.1:9000") of your running Pavlov server. Please note it is **NOT RECOMMENDED** to expose the Pavlov RCON port to the internet. It uses an insecure login mechanism and could potentially be a security risk. 
2. The Pavlov plaintext RCON password you have set up in RconSettings.ini: http://wiki.pavlov-vr.com/index.php?title=Dedicated_server#Rcon_Overview_and_Commands
3. Your **discord ID**, put discord in developer mode as shown here: https://discordia.me/en/developer-mode then right click your own portrait and "Copy ID". Something like: **735940451818481412**
4. **Discord token** go to https://discord.com/developers/applications and create a new bot and copy the bot token. Instructions for a different bot are found here. https://discordpy.readthedocs.io/en/latest/discord.html **please not this bot only requires "Send Messages and Add Reactions" permission.

Decide if $(pwd)/ivanbot is the directory you want to store bot configuration data and if not change it.
Now fill those into the following command and execute:

```
docker run -d  \
--name IvanBot \
--restart=unless-stopped \
--env IVAN_CONNECT_IP=your_ip \
--env IVAN_PASSWORD=pavlov_password \
--env ADMIN_ID=your_discord_id \
--env DISCORD_TOKEN=discord_token \
--env ALLOW_USERS=true \
-v $(pwd)/ivanbot:/root
ramoneelman/ivanbot
```

# Tutorial
All bot commands must always start with "-".

When the bot first starts it will only have 1 known admin user. You. To add more people right click on them in Discord and "Copy ID", then execute:

```-admin add [yourid]```
The same is true for adding Moderators (-mod add)

Not find some workshop maps and add them to your map pool!

```-map add https://steamcommunity.com/sharedfiles/filedetails/?id=1454448750 DM manor_ttt```

Now you'll be able to see your map pool by executing.

```-map list```

If you have enough (5) maps it's time to start a map vote.

```-map vote start 5```

Everyone can vote by adding reactions. After the vote is done (30 sec) the map will switch automatically. If you do not have any patience use ```-map vote stop```
