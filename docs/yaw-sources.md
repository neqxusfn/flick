# Preset evidence

Reviewed 11 October 2026. A preset is a conversion aid, not a claim of compatibility with a game's input handling or anti-cheat policy. Check after game updates. All values refer to normal, unscoped horizontal mouse input.

| Preset   | Degrees/count at sensitivity 1 | Input convention                      | Primary source                                                                                                                                                                                                                                             |
| -------- | ------------------------------ | ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Valorant | 0.07                           | In-game sensitivity, e.g. 0.5         | [KovaaK's developer release notes, 2.0.1.0](https://store.steampowered.com/news/posts/?appids=824270&enddate=1595643090&feed=steam_community_announcements) explicitly specify the Valorant scale.                                                         |
| Fortnite | 0.005555                       | Displayed percentage, e.g. 10 for 10% | [Same developer release notes](https://store.steampowered.com/news/posts/?appids=824270&enddate=1595643090&feed=steam_community_announcements) explicitly specify “Fortnite Percent”. This is the published precision, not an invented repeating fraction. |
| Custom   | User supplied                  | Sensitivity × user yaw                | No game value is claimed. The editable initial yaw is an example only.                                                                                                                                                                                     |

The included primary source is the sensitivity tool developer's own release announcement, not a Riot/Epic guarantee. Current game builds were not tested. Recalibrate if the turn is inaccurate.

## Omitted presets

- **CS2:** the Valve-hosted console-variable reference could not be fetched (403). Valve's [Half-Life source](https://github.com/ValveSoftware/halflife/blob/master/cl_dll/inputw32.cpp) mentions 0.022, but that does not establish the value for current CS2. No inference across games was made.
- **Apex Legends:** searched EA sources; forum discussions did not establish an authoritative current yaw.
- **Overwatch 2:** no explicit developer source establishing degrees per raw count was found.
- **Call of Duty (modern):** no explicit developer source establishing one constant across current titles and sensitivity scales was found.
- **Rainbow Six Siege:** [Ubisoft's FOV and Input Sensitivity article](https://www.ubisoft.com/en-gb/game/rainbow-six/siege/news-updates/6kY6b5JByBY3P6vQWWinla/fov-and-input-sensitivity) documents MouseSensitivityMultiplierUnit = 0.02 and relative multipliers. It does not establish the base degrees/count needed here. Treating 0.02 as the requested yaw would be misleading.

Use measured cm/360 + DPI or the Raw Input calibration helper for these games. Values from unsourced sensitivity calculators were not used.
