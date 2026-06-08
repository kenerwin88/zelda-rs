# NES_Ver2 Naming Evidence

This is the review surface for checking Rust RAM names against original NES_Ver2 labels and comments.

## Source Evidence Review Queue

These rows have useful NES_Ver2 evidence that differs from the Rust name. Original comments are preserved, and the US-English hint is a conservative glossary translation. Keep the Rust name when it is clearer; use this table to confirm meaning or catch names that are genuinely wrong.

| Address | Rust | NES_Ver2 | Original comment | US-English hint | Source |
|---|---|---|---|---|---|
| `0x00010` | `MAIN_MODULE_INDEX_TAGALONG` | `SLMODE` | select MODE | select MODE | `us_asm/zel_ram.asm:23` |
| `0x00010` | `MAIN_MODULE_INDEX` | `SLMODE` | select MODE | select MODE | `us_asm/zel_ram.asm:23` |
| `0x00011` | `SUBMODULE_INDEX_DN` | `GAMEMD` | Game Mode select no. | Game Mode select no. | `us_asm/zel_ram.asm:24` |
| `0x00011` | `SUBMODULE_INDEX_TAGALONG` | `GAMEMD` | Game Mode select no. | Game Mode select no. | `us_asm/zel_ram.asm:24` |
| `0x00011` | `SUBMODULE_INDEX` | `GAMEMD` | Game Mode select no. | Game Mode select no. | `us_asm/zel_ram.asm:24` |
| `0x00012` | `NMI_BOOLEAN` | `NMIFLG` | NMI flag | NMI flag | `us_asm/zel_ram.asm:25` |
| `0x00013` | `INIDISP_COPY` | `BLKFLG` | Blanking check flag | Blanking check flag | `us_asm/zel_ram.asm:26` |
| `0x00014` | `NMI_LOAD_BG_FROM_VRAM` | `VRFLG` | VMA pointer flag | VMA pointer flag | `us_asm/zel_ram.asm:27` |
| `0x00015` | `FLAG_UPDATE_CGRAM_IN_NMI` | `CGVMAF` | CG. VMA check flag | CG. VMA check flag | `us_asm/zel_ram.asm:28` |
| `0x00016` | `FLAG_UPDATE_HUD_IN_NMI` | `B3CHFG` | BG.3 write flag | BG.3 write flag | `us_asm/zel_ram.asm:29` |
| `0x00017` | `NMI_SUBROUTINE_INDEX` | `BGWTFG` | BG. write point flag | BG. write point flag | `us_asm/zel_ram.asm:30` |
| `0x00018` | `NMI_COPY_PACKETS_FLAG` | `BGCHFG` | BG. change write flag | BG. change write flag | `us_asm/zel_ram.asm:31` |
| `0x00019` | `NMI_UPDATE_TILEMAP_DST_LOAD_GFX` | `OBCCFG` | OBJ. BG. character change flag | OBJ. BG. character change flag | `us_asm/zel_ram.asm:32` |
| `0x00019` | `NMI_UPDATE_TILEMAP_DST` | `OBCCFG` | OBJ. BG. character change flag | OBJ. BG. character change flag | `us_asm/zel_ram.asm:32` |
| `0x0001a` | `FRAME_COUNTER_TAGALONG` | `FCNT` | frame counter | frame counter | `us_asm/zel_ram.asm:33` |
| `0x0001a` | `FRAME_COUNTER` | `FCNT` | frame counter | frame counter | `us_asm/zel_ram.asm:33` |
| `0x0001b` | `PLAYER_IS_INDOORS_TAGALONG` | `GMMODE` | ground/danjyon mode (0:ground , 1:danjyon) | ground/dungeon mode (0:ground , 1:dungeon) | `us_asm/zel_ram.asm:35` |
| `0x0001b` | `PLAYER_IS_INDOORS` | `GMMODE` | ground/danjyon mode (0:ground , 1:danjyon) | ground/dungeon mode (0:ground , 1:dungeon) | `us_asm/zel_ram.asm:35` |
| `0x0001c` | `TM_COPY` | `DPMAIN` | display mode main | display mode main | `us_asm/zel_ram.asm:37` |
| `0x0001e` | `TMW_COPY` | `WDMAIN` | window display main | window display main | `us_asm/zel_ram.asm:39` |
| `0x00020` | `LINK_Y_COORD_TAGALONG` | `PLYPS1` | Y-pos. low | Y-position. low byte | `us_asm/zel_ram.asm:45` |
| `0x00020` | `LINK_Y_COORD` | `PLYPS1` | Y-pos. low | Y-position. low byte | `us_asm/zel_ram.asm:45` |
| `0x00022` | `LINK_X_COORD_TAGALONG` | `PLXPS1` | X-pos. low | X-position. low byte | `us_asm/zel_ram.asm:47` |
| `0x00022` | `ATTRACT_STATE` | `PLXPS1` | X-pos. low | X-position. low byte | `us_asm/zel_ram.asm:47` |
| `0x00022` | `LINK_X_COORD` | `PLXPS1` | X-pos. low | X-position. low byte | `us_asm/zel_ram.asm:47` |
| `0x00023` | `ATTRACT_SEQUENCE` | `PLXPS0` | hi | high byte | `us_asm/zel_ram.asm:48` |
| `0x00024` | `LINK_Z_COORD_TAGALONG` | `PLZPS1` | Z-pos. low | Z-position. low byte | `us_asm/zel_ram.asm:49` |
| `0x00024` | `LINK_Z_COORD` | `PLZPS1` | Z-pos. low | Z-position. low byte | `us_asm/zel_ram.asm:49` |
| `0x00025` | `ATTRACT_SCENE_TIMER` | `PLZPS0` | hi | high byte | `us_asm/zel_ram.asm:50` |
| `0x00026` | `ATTRACT_NEXT_LEGEND_GFX` | `PLMUKI` | idou houkou | movement direction | `us_asm/zel_ram.asm:51` |
| `0x00026` | `LINK_DIRECTION_LAST` | `PLMUKI` | idou houkou | movement direction | `us_asm/zel_ram.asm:51` |
| `0x00027` | `ATTRACT_LEGEND_FLAG` | `PLYSPD` | Y-speed | Y-speed | `us_asm/zel_ram.asm:52` |
| `0x00027` | `LINK_ACTUAL_VEL_Y` | `PLYSPD` | Y-speed | Y-speed | `us_asm/zel_ram.asm:52` |
| `0x00028` | `ATTRACT_X_BASE` | `PLXSPD` | X-speed | X-speed | `us_asm/zel_ram.asm:53` |
| `0x00028` | `LINK_ACTUAL_VEL_X` | `PLXSPD` | X-speed | X-speed | `us_asm/zel_ram.asm:53` |
| `0x00029` | `ATTRACT_Y_BASE` | `PLZSPD` | Z-speed | Z-speed | `us_asm/zel_ram.asm:54` |
| `0x00029` | `LINK_ACTUAL_VEL_Z` | `PLZSPD` | Z-speed | Z-speed | `us_asm/zel_ram.asm:54` |
| `0x0002a` | `ATTRACT_OAM_IDX` | `PLYBUF` | Y-speed buff | Y-speed buff | `us_asm/zel_ram.asm:55` |
| `0x0002a` | `LINK_SUBPIXEL_Y` | `PLYBUF` | Y-speed buff | Y-speed buff | `us_asm/zel_ram.asm:55` |
| `0x0002b` | `ATTRACT_PRISON_ZELDA_Y_BASE` | `PLXBUF` | X-speed buff | X-speed buff | `us_asm/zel_ram.asm:56` |
| `0x0002b` | `LINK_SUBPIXEL_X` | `PLXBUF` | X-speed buff | X-speed buff | `us_asm/zel_ram.asm:56` |
| `0x0002c` | `ATTRACT_THRONE_FADE_TIMER` | `PLZBUF` | Z-speed buff | Z-speed buff | `us_asm/zel_ram.asm:57` |
| `0x0002c` | `LINK_SUBPIXEL_Z` | `PLZBUF` | Z-speed buff | Z-speed buff | `us_asm/zel_ram.asm:57` |
| `0x0002d` | `ATTRACT_VAR7` | `PYFLCH` | player flem chenge | player frame change | `us_asm/zel_ram.asm:58` |
| `0x0002d` | `LINK_FRAME_CHANGE_COUNTER` | `PYFLCH` | player flem chenge | player frame change | `us_asm/zel_ram.asm:58` |
| `0x0002e` | `LINK_ANIMATION_STEPS` | `PYCRCH` | player chara chenge | player character change | `us_asm/zel_ram.asm:59` |
| `0x0002f` | `LINK_DIRECTION_FACING_TAGALONG` | `PLMKCH` | player chara muki | player character direction | `us_asm/zel_ram.asm:60` |
| `0x0002f` | `LINK_DIRECTION_FACING` | `PLMKCH` | player chara muki | player character direction | `us_asm/zel_ram.asm:60` |
| `0x00030` | `LINK_Y_VEL_TAGALONG` | `PLYMVC` | Y-pos dyoo | Y-position state | `us_asm/zel_ram.asm:61` |
| `0x00030` | `ATTRACT_VRAM_DST` | `PLYMVC` | Y-pos dyoo | Y-position state | `us_asm/zel_ram.asm:61` |
| `0x00030` | `LINK_Y_VEL` | `PLYMVC` | Y-pos dyoo | Y-position state | `us_asm/zel_ram.asm:61` |
| `0x00031` | `LINK_X_VEL_TAGALONG` | `PLXMVC` | X-pos dyoo | X-position state | `us_asm/zel_ram.asm:62` |
| `0x00031` | `LINK_X_VEL` | `PLXMVC` | X-pos dyoo | X-position state | `us_asm/zel_ram.asm:62` |
| `0x00032` | `ATTRACT_ANIM_STEP_COUNTER` | `PLHYBF0` | jump hozon work (l) | jump saved work (l) | `us_asm/zel_ram.asm:63` |
| `0x00033` | `ATTRACT_SOLDIER_ANIM_STEP` | `PLHYBF1` | jump hozon work (h) | jump saved work (h) | `us_asm/zel_ram.asm:64` |
| `0x00034` | `ATTRACT_PRISON_SOLDIER_X_LO` | `SPYPS` | next Y-pos (L) | next Y-position (L) | `us_asm/zel_ram.asm:65` |
| `0x00038` | `TILEDETECT_DIAGONAL_TILE` | `BGCRNO` | special BG sub flag (L) | special BG sub flag (L) | `us_asm/zel_ram.asm:69` |
| `0x0003a` | `BUTTON_MASK_B_Y_TAGALONG` | `KENKY` | ken push key flag | sword push key flag | `us_asm/zel_ram.asm:71` |
| `0x0003a` | `BUTTON_MASK_B_Y` | `KENKY` | ken push key flag | sword push key flag | `us_asm/zel_ram.asm:71` |
| `0x0003b` | `Y_BUTTON_ACTION_FLAGS` | `KENKYL` | y key flag | y key flag | `us_asm/zel_ram.asm:72` |
| `0x0003c` | `BUTTON_B_FRAMES` | `KENMD` | ken mode | sword mode | `us_asm/zel_ram.asm:73` |
| `0x0003d` | `LINK_DELAY_TIMER_SPIN_ATTACK` | `KENFM` | ken flem counter | sword frame counter | `us_asm/zel_ram.asm:74` |
| `0x0003e` | `LINK_Y_COORD_SAFE_RETURN_LO` | `PLYHN0` | Y-pos hozon (H) | Y-position saved (H) | `us_asm/zel_ram.asm:75` |
| `0x0003f` | `LINK_X_COORD_SAFE_RETURN_LO` | `PLXHN0` | X-pos hozon (H) | X-position saved (H) | `us_asm/zel_ram.asm:76` |
| `0x00040` | `ATTRACT_X_BASE_HI` | `PLYHN1` | Y-pos hozon (L) | Y-position saved (L) | `us_asm/zel_ram.asm:77` |
| `0x00040` | `LINK_Y_COORD_SAFE_RETURN_HI` | `PLYHN1` | Y-pos hozon (L) | Y-position saved (L) | `us_asm/zel_ram.asm:77` |
| `0x00041` | `LINK_X_COORD_SAFE_RETURN_HI` | `PLXHN1` | X-pos hozon (L) | X-position saved (L) | `us_asm/zel_ram.asm:78` |
| `0x00042` | `LINK_DIRECTION_MASK_A` | `PLMVKY` | tate key flag | vertical key flag | `us_asm/zel_ram.asm:79` |
| `0x00043` | `LINK_DIRECTION_MASK_B` | `PLMVKY1` | yoko key flag | horizontal key flag | `us_asm/zel_ram.asm:80` |
| `0x00044` | `PLAYER_OAM_Y_OFFSET` | `KNCRYP` | enmy ken haba y-pos | enmy sword haba y-position | `us_asm/zel_ram.asm:81` |
| `0x00045` | `PLAYER_OAM_X_OFFSET` | `KNCRXP` | enmy ken haba x-pos | enmy sword haba x-position | `us_asm/zel_ram.asm:82` |
| `0x00046` | `LINK_INCAPACITATED_TIMER` | `HANEFG` | hanekaeri flag | hanekaeri flag | `us_asm/zel_ram.asm:83` |
| `0x00047` | `SET_WHEN_DAMAGING_ENEMIES` | `HANIFG` | In fight area flag | In fight area flag | `us_asm/zel_ram.asm:84` |
| `0x00048` | `PLAYER_DEFENSE_FLAGS` | `HANIFG1` | ken difence flag | sword difence flag | `us_asm/zel_ram.asm:85` |
| `0x00049` | `FORCE_MOVE_ANY_DIRECTION` | `DRATMV` | door auto move flag (L) | door auto move flag (L) | `us_asm/zel_ram.asm:86` |
| `0x0004b` | `LINK_VISIBILITY_STATUS` | `OMSBMD` | oam sub mode flag | oam sub mode flag | `us_asm/zel_ram.asm:88` |
| `0x0004c` | `CAPE_DECREMENT_COUNTER` | `KAKUFM` | kakuremino flem flag | cape frame flag | `us_asm/zel_ram.asm:89` |
| `0x0004d` | `LINK_AUXILIARY_STATE_TAGALONG` | `DIEFG` | die flag | die flag | `us_asm/zel_ram.asm:90` |
| `0x0004d` | `LINK_AUXILIARY_STATE` | `DIEFG` | die flag | die flag | `us_asm/zel_ram.asm:90` |
| `0x0004e` | `DUNG_TRANSITION_LANDING_CLASS` | `PYATFG` | auto flag | auto flag | `us_asm/zel_ram.asm:91` |
| `0x0004f` | `INDEX_OF_DASHING_SFX` | `DSTMFM` | dash sound timer flem conter | dash sound timer frame conter | `us_asm/zel_ram.asm:92` |
| `0x00050` | `ATTRACT_SCENE_FRAME_COUNTER` | `PYMKFG` | muki kotei flag | direction fixed flag | `us_asm/zel_ram.asm:93` |
| `0x00050` | `LINK_CANT_CHANGE_DIRECTION` | `PYMKFG` | muki kotei flag | direction fixed flag | `us_asm/zel_ram.asm:93` |
| `0x00051` | `ATTRACT_LOW_RAM_CLEAR_LEN` | `PLHNL0` | y-pos hozon (l) 0 | y-position saved (l) 0 | `us_asm/zel_ram.asm:94` |
| `0x00051` | `ATTRACT_MAIDEN_WARP_STEP` | `PLHNL0` | y-pos hozon (l) 0 | y-position saved (l) 0 | `us_asm/zel_ram.asm:94` |
| `0x00051` | `TILEDETECT_WHICH_Y_POS` | `PLHNL0` | y-pos hozon (l) 0 | y-position saved (l) 0 | `us_asm/zel_ram.asm:94` |
| `0x00052` | `ATTRACT_FADE_IN_COMPLETE_FLAG` | `PLHNH0` | y-pos hozon (H) 0 | y-position saved (H) 0 | `us_asm/zel_ram.asm:95` |
| `0x00055` | `LINK_CAPE_MODE` | `KAKUMD` | kakuremino mode | cape mode | `us_asm/zel_ram.asm:98` |
| `0x00056` | `LINK_IS_BUNNY` | `RABIFG` | rabit hozon flag | rabit saved flag | `us_asm/zel_ram.asm:99` |
| `0x00057` | `LINK_SPEED_MODIFIER` | `PSTPFG` | step check flag | step check flag | `us_asm/zel_ram.asm:100` |
| `0x00058` | `TILEDETECT_STAIR_TILE` | `KDFGST` | kaidan BG check flag | stairs BG check flag | `us_asm/zel_ram.asm:101` |
| `0x00059` | `TILEDETECT_PIT_TILE` | `HOLEFG` | hole BG check flag | hole BG check flag | `us_asm/zel_ram.asm:102` |
| `0x0005a` | `PLAYER_PIT_DATA_INDEX` | `HOLEFG1` | hole data index flag | hole data index flag | `us_asm/zel_ram.asm:103` |
| `0x0005b` | `PLAYER_NEAR_PIT_STATE_TAGALONG` | `HOLEFG2` | hole mode flag | hole mode flag | `us_asm/zel_ram.asm:104` |
| `0x0005b` | `PLAYER_NEAR_PIT_STATE` | `HOLEFG2` | hole mode flag | hole mode flag | `us_asm/zel_ram.asm:104` |
| `0x0005c` | `LINK_SPRITE_OAM_STATE_TIMER` | `PLHLFM` | hole flem counter | hole frame counter | `us_asm/zel_ram.asm:105` |
| `0x0005d` | `LINK_PLAYER_HANDLER_STATE` | `LNMODE` | mode flag | mode flag | `us_asm/zel_ram.asm:106` |
| `0x0005d` | `LINK_PLAYER_HANDLER_STATE_TAGALONG` | `LNMODE` | mode flag | mode flag | `us_asm/zel_ram.asm:106` |
| `0x0005d` | `ATTRACT_SCENE_DONE_FLAG` | `LNMODE` | mode flag | mode flag | `us_asm/zel_ram.asm:106` |
| `0x0005d` | `LINK_PLAYER_HANDLER_STATE` | `LNMODE` | mode flag | mode flag | `us_asm/zel_ram.asm:106` |
| `0x0005e` | `LINK_SPEED_SETTING_TAGALONG` | `PYSPFG` | speed index flag | speed index flag | `us_asm/zel_ram.asm:107` |
| `0x0005e` | `LINK_SPEED_SETTING` | `PYSPFG` | speed index flag | speed index flag | `us_asm/zel_ram.asm:107` |
| `0x0005f` | `ATTRACT_FADE_IN_DONE_FLAG` | `BKONFG` | block bit on flag (L) | block bit on flag (L) | `us_asm/zel_ram.asm:108` |
| `0x0005f` | `TILEDETECT_BLOCK_FLAGS_LO` | `BKONFG` | block bit on flag (L) | block bit on flag (L) | `us_asm/zel_ram.asm:108` |
| `0x00060` | `ATTRACT_SCENE_SUBSTEP` | `BKONFG1` | block bit on flag (H) | block bit on flag (H) | `us_asm/zel_ram.asm:109` |
| `0x00061` | `ATTRACT_SUBSTEP_DELAY_COUNTER` | `BLKFLM` | block wait flem counter | block wait frame counter | `us_asm/zel_ram.asm:110` |
| `0x00061` | `GRAVESTONE_PUSH_TIMEOUT` | `BLKFLM` | block wait flem counter | block wait frame counter | `us_asm/zel_ram.asm:110` |
| `0x00062` | `ATTRACT_MAIDEN_WARP_TIMER_A` | `DRMKFG` | door muki flag | door direction flag | `us_asm/zel_ram.asm:111` |
| `0x00062` | `TILEDETECT_DOOR_DIRECTION_FLAGS` | `DRMKFG` | door muki flag | door direction flag | `us_asm/zel_ram.asm:111` |
| `0x00063` | `ATTRACT_MAIDEN_WARP_TIMER_B` | `DRMKFG1` | door muki flag 1 | door direction flag 1 | `us_asm/zel_ram.asm:112` |
| `0x00064` | `OAM_PRIORITY_VALUE_TAGALONG` | `PYBGUN` | BG uusen juni flag | BG priority order flag | `us_asm/zel_ram.asm:113` |
| `0x00064` | `OAM_PRIORITY_VALUE` | `PYBGUN` | BG uusen juni flag | BG priority order flag | `us_asm/zel_ram.asm:113` |
| `0x00072` | `SCRATCH_0_ANCILLA` | `BMWORK` | beam work | beam work | `us_asm/zel_ram.asm:128` |
| `0x00072` | `SCRATCH_0` | `BMWORK` | beam work | beam work | `us_asm/zel_ram.asm:128` |
| `0x00073` | `SCRATCH_A` | `CRTNL` | certen left | certen left | `us_asm/zel_label.asm:263` |
| `0x00074` | `SCRATCH_1_ANCILLA` | `CRTNR` | right | right | `us_asm/zel_label.asm:264` |
| `0x00074` | `SCRATCH_1` | `CRTNR` | right | right | `us_asm/zel_label.asm:264` |
| `0x00076` | `INDEX_OF_INTERACTING_TILE_ANCILLA` | `LWNDW` | window left | window left | `us_asm/zel_label.asm:266` |
| `0x00076` | `INDEX_OF_INTERACTING_TILE` | `LWNDW` | window left | window left | `us_asm/zel_label.asm:266` |
| `0x00078` | `ALLOW_SCROLL_Z` | `PTBIFG` | player tobi flag | player tobi flag | `us_asm/zel_ram.asm:131` |
| `0x00079` | `LINK_SPIN_ATTACK_STEP_COUNTER` | `KENTIM` | player ken kaiten timer | player sword kaiten timer | `us_asm/zel_ram.asm:132` |
| `0x0007b` | `LAST_LIGHT_VS_DARK_WORLD` | `OMPHZN` | player omote,ura hozon flag | player omote,ura saved flag | `us_asm/zel_ram.asm:133` |
| `0x00084` | `MAP16_LOAD_SRC_OFF_OVERWORLD` | `SCRPNT` | screen no kihon no iti | screen no base no position | `us_asm/zel_ram.asm:136` |
| `0x00086` | `MAP16_LOAD_DST_OFF_OVERWORLD` | `XWRITE` | yoko no unit no iti | horizontal no unit no position | `us_asm/zel_ram.asm:137` |
| `0x00088` | `MAP16_LOAD_Y_UNIT_OVERWORLD` | `YWRITE` | tate no unit no iti | vertical no unit no position | `us_asm/zel_ram.asm:138` |
| `0x0008a` | `OVERWORLD_SCREEN_INDEX_TAGALONG` | `MPDTNO` | dono map ka o simesu 0--8 | dono map ka o simesu 0--8 | `us_asm/zel_ram.asm:139` |
| `0x0008a` | `OVERWORLD_SCREEN_INDEX` | `MPDTNO` | dono map ka o simesu 0--8 | dono map ka o simesu 0--8 | `us_asm/zel_ram.asm:139` |
| `0x00090` | `OAM_CUR_PTR_TAGALONG` | `OAMADR` | oam address | oam address | `us_asm/zel_ram.asm:144` |
| `0x00090` | `OAM_CUR_PTR` | `OAMADR` | oam address | oam address | `us_asm/zel_ram.asm:144` |
| `0x00092` | `OAM_EXT_CUR_PTR_TAGALONG` | `OSBADR` | oam-sub address | oam-sub address | `us_asm/zel_ram.asm:145` |
| `0x00092` | `OAM_EXT_CUR_PTR` | `OSBADR` | oam-sub address | oam-sub address | `us_asm/zel_ram.asm:145` |
| `0x0009d` | `COLDATA_COPY1` | `WD2132G` | (green) | (green) | `us_asm/zel_ram.asm:156` |
| `0x0009e` | `COLDATA_COPY2` | `WD2132B` | (blue) | (blue) | `us_asm/zel_ram.asm:157` |
| `0x000a0` | `DUNGEON_ROOM_INDEX_TAGALONG` | `RMXYCT` | room x,y-counter | room x,y-counter | `us_asm/zel_ram.asm:159` |
| `0x000a0` | `DUNGEON_ROOM_INDEX` | `RMXYCT` | room x,y-counter | room x,y-counter | `us_asm/zel_ram.asm:159` |
| `0x000a2` | `DUNGEON_ROOM_INDEX_PREV` | `BKRMPT` | befoer room pointer | befoer room pointer | `us_asm/zel_ram.asm:160` |
| `0x000a4` | `DUNG_CUR_FLOOR` | `FLORNO` | floor no. | floor no. | `us_asm/zel_ram.asm:161` |
| `0x000a6` | `QUADRANT_FULLSIZE_X` | `RMCKXF` | x-check flag | x-check flag | `us_asm/zel_ram.asm:162` |
| `0x000a7` | `QUADRANT_FULLSIZE_Y` | `RMCKYF` | y-check flag | y-check flag | `us_asm/zel_ram.asm:163` |
| `0x000a8` | `COMPOSITE_OF_LAYOUT_AND_QUADRANT` | `RMCKPT` | check pointer | check pointer | `us_asm/zel_ram.asm:164` |
| `0x000a9` | `LINK_QUADRANT_X` | `RMXCPT` | x-check pointer | x-check pointer | `us_asm/zel_ram.asm:165` |
| `0x000ad` | `DUNG_HDR_COLLISION_2` | `BGMVFG` | BG. move data flag | BG. move data flag | `us_asm/zel_ram.asm:169` |
| `0x000ae` | `DUNG_HDR_TAG` | `INFDF0` | information data flag-0 | information data flag-0 | `us_asm/zel_ram.asm:170` |
| `0x000b0` | `SUBSUBMODULE_INDEX` | `JRSBPT` | JSRSUB pointer | JSRSUB pointer | `us_asm/zel_ram.asm:173` |
| `0x000b2` | `DUNG_DRAW_WIDTH_INDICATOR` | `XSTCNT` | x-set counter | x-set counter | `us_asm/zel_ram.asm:174` |
| `0x000b4` | `DUNG_DRAW_HEIGHT_INDICATOR` | `YSTCNT` | y-set | y-set | `us_asm/zel_ram.asm:176` |
| `0x000b7` | `DUNG_LOAD_PTR` | `DTBFWK` | data buffer work | data buffer work | `us_asm/zel_ram.asm:179` |
| `0x000b9` | `DUNG_LOAD_PTR_BANK` | `CNGY5` | change-yuka (B) | change-floor (B) | `us_asm/zel_label.asm:337` |
| `0x000ba` | `DUNG_LOAD_PTR_OFFS` | `DTRDPT` | data read pointer | data read pointer | `us_asm/zel_ram.asm:180` |
| `0x000bd` | `HUD_TMP1` | `WORKZ` | work area | work area | `us_asm/zel_ram.asm:182` |
| `0x000c7` | `LINK_RECOIL_Z_VEL` | `STPYK` | stop yuka | stop floor | `us_asm/zel_label.asm:353` |
| `0x000e0` | `BG1HOFS_COPY2` | `SCCH1` | BG-1. H-scroll counter_L | BG-1. H-scroll counter_L | `us_asm/zel_ram.asm:213` |
| `0x000e6` | `BG1VOFS_COPY2` | `SCCV1` | BG-1. V-scroll counter_L | BG-1. V-scroll counter_L | `us_asm/zel_ram.asm:219` |
| `0x000ec` | `TILEMAP_LOCATION_CALC_MASK` | `PSCKRM` | position check ram (G=03F8H , D=01F8H) | position check ram (G=03F8H , D=01F8H) | `us_asm/zel_ram.asm:226` |
| `0x000ee` | `LINK_IS_ON_LOWER_LEVEL_TAGALONG` | `PLBGCKF` | player BG. check flag (0:BG2 , 1:BG1) | player BG. check flag (0:BG2 , 1:BG1) | `us_asm/zel_ram.asm:227` |
| `0x000ee` | `LINK_IS_ON_LOWER_LEVEL` | `PLBGCKF` | player BG. check flag (0:BG2 , 1:BG1) | player BG. check flag (0:BG2 , 1:BG1) | `us_asm/zel_ram.asm:227` |
| `0x000ef` | `ROOM_TRANSITIONING_FLAGS` | `PLMDCCF` | mode change check flag | mode change check flag | `us_asm/zel_ram.asm:228` |
| `0x000f0` | `JOYPAD1H_LAST` | `KEYA1` | key repeat (A,B,sl,st,u,d,l,r) | key repeat (A,B,sl,st,u,d,l,r) | `us_asm/zel_ram.asm:231` |
| `0x000f4` | `FILTERED_JOYPAD_H` | `KEYA2` | triga | triga | `us_asm/zel_ram.asm:235` |
| `0x000f8` | `JOYPAD1H_LAST2` | `KEYBF` | back_up | back_up | `us_asm/zel_ram.asm:239` |
| `0x000fc` | `RESET_XY_CHECK_FLAGS` | `RSXYCKF` | reset x,y check flag | reset x,y check flag | `us_asm/zel_ram.asm:242` |
| `0x000ff` | `VIRQ_TRIGGER` | `POLYTIME` | poly-gon time | poly-gon time | `us_asm/zel_ram.asm:245` |
| `0x00100` | `POLY_THREAD_RAM_LEN` | `PCHPT0` | player character pointer | player character pointer | `us_asm/zel_ram.asm:248` |
| `0x00100` | `LINK_DMA_GRAPHICS_INDEX` | `PCHPT0` | player character pointer | player character pointer | `us_asm/zel_ram.asm:248` |
| `0x00107` | `LINK_DMA_SWORD_GRAPHICS_INDEX` | `KENCPT` | sword | sword | `us_asm/zel_ram.asm:252` |
| `0x00108` | `LINK_DMA_SHIELD_GRAPHICS_INDEX` | `TATCPT` | shild | shild | `us_asm/zel_ram.asm:253` |
| `0x0010a` | `GAME_OVER_CHECK_FLAG` | `GOVRCFG` | game-over check flag | game-over check flag | `us_asm/zel_ram.asm:257` |
| `0x0010a` | `GAME_OVER_CHECK_FLAG` | `GOVRCFG` | game-over check flag | game-over check flag | `us_asm/zel_ram.asm:257` |
| `0x0010c` | `SAVED_MODULE_FOR_MENU` | `NXSLMD` | next select MODE | next select MODE | `us_asm/zel_ram.asm:259` |
| `0x0010c` | `SAVED_MODULE_FOR_MENU_TAGALONG` | `NXSLMD` | next select MODE | next select MODE | `us_asm/zel_ram.asm:259` |
| `0x0010c` | `SAVED_MODULE_FOR_MENU` | `NXSLMD` | next select MODE | next select MODE | `us_asm/zel_ram.asm:259` |
| `0x00110` | `DUNG_INDEX_X3` | `RMDTPT` | room data pointer | room data pointer | `us_asm/zel_ram.asm:263` |
| `0x00112` | `FLAG_CUSTOM_SPELL_ANIM_ACTIVE` | `SPMCFG` | special magic flag | special magic flag | `us_asm/zel_ram.asm:265` |
| `0x00114` | `LINK_TILE_BELOW` | `EXITFG` | exit-door flag | exit-door flag | `us_asm/zel_ram.asm:267` |
| `0x00118` | `NMI_UPDATE_TILEMAP_SRC_LOAD_GFX` | `CHADRF` | chracter change Vram address | chracter change Vram address | `us_asm/zel_ram.asm:273` |
| `0x00118` | `NMI_UPDATE_TILEMAP_SRC` | `CHADRF` | chracter change Vram address | chracter change Vram address | `us_asm/zel_ram.asm:273` |
| `0x0011a` | `BG1_X_OFFSET` | `YUREXD` | yure x-data | yure x-data | `us_asm/zel_ram.asm:275` |
| `0x0011c` | `BG1_Y_OFFSET` | `YUREYD` | y-data | y-data | `us_asm/zel_ram.asm:276` |
| `0x0011e` | `BG2HOFS_COPY` | `SSCCH2` | set SCCH2 | set SCCH2 | `us_asm/zel_ram.asm:277` |
| `0x00120` | `BG1HOFS_COPY` | `SSCCH1` | SCCH1 | SCCH1 | `us_asm/zel_ram.asm:278` |
| `0x00122` | `BG2VOFS_COPY` | `SSCCV2` | SCCV2 | SCCV2 | `us_asm/zel_ram.asm:279` |
| `0x00124` | `BG1VOFS_COPY` | `SSCCV1` | SCCV1 | SCCV1 | `us_asm/zel_ram.asm:280` |
| `0x00126` | `TRANSITION_COUNTER` | `PSCCCT` | player scroll counter | player scroll counter | `us_asm/zel_ram.asm:282` |
| `0x00126` | `TRANSITION_COUNTER_OVERWORLD` | `PSCCCT` | player scroll counter | player scroll counter | `us_asm/zel_ram.asm:282` |
| `0x00128` | `IRQ_FLAG` | `IRQSWFG` | IRQ swich flag | IRQ swich flag | `us_asm/zel_ram.asm:284` |
| `0x0012a` | `IS_NMI_THREAD_ACTIVE` | `POLYCFG` | polygon check flag | polygon check flag | `us_asm/zel_ram.asm:286` |
| `0x0012c` | `MUSIC_CONTROL_TAGALONG` | `SOUND0` | sound port-0 | sound port-0 | `us_asm/zel_ram.asm:288` |
| `0x0012c` | `MUSIC_CONTROL` | `SOUND0` | sound port-0 | sound port-0 | `us_asm/zel_ram.asm:288` |
| `0x00130` | `CURRENT_MUSIC_CONTROL` | `SVSND0` | check | check | `us_asm/zel_ram.asm:292` |
| `0x00131` | `SOUND_EFFECT_AMBIENT_LAST` | `SVCKF1` | check | check | `us_asm/zel_ram.asm:293` |
| `0x00132` | `QUEUED_MUSIC_CONTROL` | `SNDSFG0` | sound0 set flag | sound0 set flag | `us_asm/zel_ram.asm:294` |
| `0x00133` | `LAST_MUSIC_CONTROL` | `SVCKF0` | check flag | check flag | `us_asm/zel_ram.asm:295` |
| `0x00134` | `ANIMATED_TILE_VRAM_ADDR` | `WTRADR` | water write address | water write address | `us_asm/zel_ram.asm:297` |
| `0x00136` | `FLAG_WHICH_MUSIC_TYPE_DUNGEON` | `SNDPCFG` | sound program check flag | sound program check flag | `us_asm/zel_ram.asm:299` |
| `0x00136` | `FLAG_WHICH_MUSIC_TYPE` | `SNDPCFG` | sound program check flag | sound program check flag | `us_asm/zel_ram.asm:299` |
| `0x00136` | `FLAG_WHICH_MUSIC_TYPE_MESSAGING` | `SNDPCFG` | sound program check flag | sound program check flag | `us_asm/zel_ram.asm:299` |
| `0x00200` | `ATTRACT_LEGEND_CTR` | `MAPDMD` | map display mode | map display mode | `us_asm/zel_ram.asm:306` |
| `0x00200` | `OVERWORLD_MAP_STATE` | `MAPDMD` | map display mode | map display mode | `us_asm/zel_ram.asm:306` |
| `0x00202` | `HUD_CUR_ITEM` | `GETITM0` | B no mmoteiru yatu | B no mmoteiru yatu | `us_asm/zel_ram.asm:307` |
| `0x00205` | `BOTTLE_MENU_EXPAND_ROW` | `MAPSCT` | map scroll counter | map scroll counter | `us_asm/zel_ram.asm:312` |

## Weak Rust Names With Source Evidence

| Address | Rust | NES_Ver2 | Original comment | US-English hint | Source |
|---|---|---|---|---|---|
| `0x00072` | `SCRATCH_0_ANCILLA` | `BMWORK` | beam work | beam work | `us_asm/zel_ram.asm:128` |
| `0x00072` | `SCRATCH_0` | `BMWORK` | beam work | beam work | `us_asm/zel_ram.asm:128` |
| `0x00073` | `SCRATCH_A` | `CRTNL` | certen left | certen left | `us_asm/zel_label.asm:263` |
| `0x00074` | `SCRATCH_1_ANCILLA` | `CRTNR` | right | right | `us_asm/zel_label.asm:264` |
| `0x00074` | `SCRATCH_1` | `CRTNR` | right | right | `us_asm/zel_label.asm:264` |

## Ambiguous Address Matches

| Address | Rust | Chosen label | Match count |
|---|---|---|---|
| `0x00000` | `R0` | `WORK` | 61 |
| `0x00002` | `R2` | `WORK2` | 23 |
| `0x00006` | `R6` | `WORK6` | 21 |
| `0x0000a` | `R10` | `BMNO` | 25 |
| `0x0000c` | `R12` | `WORKC` | 20 |
| `0x0000e` | `R14` | `WORKE` | 17 |
| `0x0000f` | `R15` | `WORKF` | 24 |
| `0x00010` | `MAIN_MODULE_INDEX_TAGALONG` | `SLMODE` | 22 |
| `0x00010` | `MAIN_MODULE_INDEX` | `SLMODE` | 22 |
| `0x00011` | `SUBMODULE_INDEX_DN` | `GAMEMD` | 16 |
| `0x00011` | `SUBMODULE_INDEX_TAGALONG` | `GAMEMD` | 16 |
| `0x00011` | `SUBMODULE_INDEX` | `GAMEMD` | 16 |
| `0x00012` | `NMI_BOOLEAN` | `NMIFLG` | 19 |
| `0x00013` | `INIDISP_COPY` | `BLKFLG` | 22 |
| `0x00014` | `NMI_LOAD_BG_FROM_VRAM` | `VRFLG` | 20 |
| `0x00015` | `FLAG_UPDATE_CGRAM_IN_NMI` | `CGVMAF` | 18 |
| `0x00016` | `FLAG_UPDATE_HUD_IN_NMI` | `B3CHFG` | 18 |
| `0x00017` | `NMI_SUBROUTINE_INDEX` | `BGWTFG` | 20 |
| `0x00018` | `NMI_COPY_PACKETS_FLAG` | `BGCHFG` | 19 |
| `0x00019` | `NMI_UPDATE_TILEMAP_DST_LOAD_GFX` | `OBCCFG` | 17 |
| `0x00019` | `NMI_UPDATE_TILEMAP_DST` | `OBCCFG` | 17 |
| `0x0001a` | `FRAME_COUNTER_TAGALONG` | `FCNT` | 20 |
| `0x0001a` | `FRAME_COUNTER` | `FCNT` | 20 |
| `0x0001b` | `PLAYER_IS_INDOORS_TAGALONG` | `GMMODE` | 18 |
| `0x0001b` | `PLAYER_IS_INDOORS` | `GMMODE` | 18 |
| `0x0001c` | `TM_COPY` | `DPMAIN` | 20 |
| `0x0001d` | `TS_COPY` | `DPSUB` | 14 |
| `0x0001e` | `TMW_COPY` | `WDMAIN` | 18 |
| `0x0001f` | `TSW_COPY` | `WDSUB` | 15 |
| `0x00020` | `ATTRACT_LOW_RAM_CLEAR_START` | `ZWORK` | 20 |
| `0x00020` | `LINK_Y_COORD_TAGALONG` | `PLYPS1` | 20 |
| `0x00020` | `ATTRACT_BG2_VOFS_BACKUP` | `ZWORK` | 20 |
| `0x00020` | `LINK_Y_COORD` | `PLYPS1` | 20 |
| `0x00022` | `LINK_X_COORD_TAGALONG` | `PLXPS1` | 16 |
| `0x00022` | `ATTRACT_STATE` | `PLXPS1` | 16 |
| `0x00022` | `LINK_X_COORD` | `PLXPS1` | 16 |
| `0x00023` | `ATTRACT_SEQUENCE` | `PLXPS0` | 15 |
| `0x00024` | `LINK_Z_COORD_TAGALONG` | `PLZPS1` | 16 |
| `0x00024` | `LINK_Z_COORD` | `PLZPS1` | 16 |
| `0x00025` | `ATTRACT_SCENE_TIMER` | `PLZPS0` | 13 |
| `0x00026` | `ATTRACT_NEXT_LEGEND_GFX` | `PLMUKI` | 14 |
| `0x00026` | `LINK_DIRECTION_LAST` | `PLMUKI` | 14 |
| `0x00027` | `ATTRACT_LEGEND_FLAG` | `PLYSPD` | 13 |
| `0x00027` | `LINK_ACTUAL_VEL_Y` | `PLYSPD` | 13 |
| `0x00028` | `ATTRACT_X_BASE` | `PLXSPD` | 14 |
| `0x00028` | `LINK_ACTUAL_VEL_X` | `PLXSPD` | 14 |
| `0x00029` | `ATTRACT_Y_BASE` | `PLZSPD` | 13 |
| `0x00029` | `LINK_ACTUAL_VEL_Z` | `PLZSPD` | 13 |
| `0x0002a` | `ATTRACT_OAM_IDX` | `PLYBUF` | 13 |
| `0x0002a` | `LINK_SUBPIXEL_Y` | `PLYBUF` | 13 |

## Source Coverage

- Source addresses mined: 1218 unique offsets
- Rust constants with at least one source match: 1130
- Source evidence review rows shown: 200 of 761
- Weak Rust names with source evidence: 5
