# Credits

## The apps

Every app in OpenCraft Studio was made by the **ArtCraft team and their contributors**, and is
published by them under MIT OR Apache-2.0. This fork only changed what [CHANGES.md](CHANGES.md) lists.

| App | Original repository | Copyright |
|---|---|---|
| CADCraft | [storytold/cadcraft](https://github.com/storytold/cadcraft) | © 2026 ArtCraft Team and the CADCraft contributors |
| DesignCraft | [storytold/designcraft](https://github.com/storytold/designcraft) | © 2026 ArtCraft Team and the DesignCraft contributors |
| EffectCraft | [storytold/effectcraft](https://github.com/storytold/effectcraft) | © 2026 ArtCraft Team and the EffectCraft contributors |
| FilmCraft | [storytold/filmcraft](https://github.com/storytold/filmcraft) | © 2026 ArtCraft Team and the FilmCraft contributors |
| GridCraft | [storytold/gridcraft](https://github.com/storytold/gridcraft) | © 2026 ArtCraft Team and the GridCraft contributors |
| LightCraft | [storytold/lightcraft](https://github.com/storytold/lightcraft) | © 2026 ArtCraft Team and the LightCraft contributors |
| PdfCraft | [storytold/pdfcraft](https://github.com/storytold/pdfcraft) | © 2026 ArtCraft Team and the PdfCraft contributors |
| PhotoCraft | [storytold/photocraft](https://github.com/storytold/photocraft) | © 2026 ArtCraft Team and the PhotoCraft contributors |
| VectorCraft | [storytold/vectorcraft](https://github.com/storytold/vectorcraft) | © 2026 ArtCraft Team and the VectorCraft contributors |
| WordCraft | [storytold/wordcraft](https://github.com/storytold/wordcraft) | © 2026 ArtCraft Team and the WordCraft contributors |

Each app's contributors are listed in its `contributors/contributors.json` and in its
*Help ▸ About ▸ Contributors* tab. The shared libraries the apps build on come from
[storytold/craft-libs](https://github.com/storytold/craft-libs), and the optional CJK fonts from
[storytold/craft-fonts](https://github.com/storytold/craft-fonts).

The ArtCraft name and logos are trademarks of the ArtCraft Team. They are not used in this fork,
which is not affiliated with or endorsed by the ArtCraft team.

## Fonts, icons, images and other assets

Every asset in an app keeps its own licence, listed with its author and source in that app's
attribution file:

| App | Attribution |
|---|---|
| CADCraft | [cadcraft/ATTRIBUTION.md](cadcraft/ATTRIBUTION.md) |
| DesignCraft | [designcraft/ASSETS.md](designcraft/ASSETS.md) |
| EffectCraft | [effectcraft/ATTRIBUTION.md](effectcraft/ATTRIBUTION.md) |
| FilmCraft | [filmcraft/ATTRIBUTION.md](filmcraft/ATTRIBUTION.md) |
| GridCraft | [gridcraft/ATTRIBUTION.md](gridcraft/ATTRIBUTION.md) |
| LightCraft | [lightcraft/assets/ATTRIBUTION.md](lightcraft/assets/ATTRIBUTION.md) |
| PdfCraft | [pdfcraft/ATTRIBUTION.md](pdfcraft/ATTRIBUTION.md) |
| PhotoCraft | [photocraft/ATTRIBUTION.md](photocraft/ATTRIBUTION.md) |
| VectorCraft | [vectorcraft/ASSETS.md](vectorcraft/ASSETS.md) |
| WordCraft | [wordcraft/ATTRIBUTION.md](wordcraft/ATTRIBUTION.md) |

## Bundled in the installers by this fork

| What | Where | Author | Licence |
|---|---|---|---|
| Place names (`cities15000`, `admin1CodesASCII`, `countryInfo`) | LightCraft, `geonames\` | [GeoNames](https://www.geonames.org) | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); attribution in `geonames\ATTRIBUTION.txt` |
| MobileSAM (Roto Brush model) | EffectCraft, `models\` | Chaoning Zhang et al., building on Meta AI's Segment Anything and Microsoft's TinyViT ([ChaoningZhang/MobileSAM](https://github.com/ChaoningZhang/MobileSAM)) | Apache-2.0; notice in `models\mobile_sam.pt.NOTICE.txt` |
| MediaPipe Face Landmarker (face-tracking model) | EffectCraft, `models\` | Google LLC ([MediaPipe Face Landmarker](https://developers.google.com/edge/mediapipe/solutions/vision/face_landmarker)) | Apache-2.0; notice in `models\face_landmarker.task.NOTICE.txt` |

The data and models are fetched and checked (SHA-256 for the models) when the installers are
built; they are not stored in this repository.

## The installer

The one-file installer is built with the [WiX Toolset](https://wixtoolset.org) v5 (Microsoft
Reciprocal License). Its program-picker page (`installer/theme.xml`) is adapted from WiX's
standard bootstrapper application *HyperlinkTheme* (© .NET Foundation and contributors, MS-RL;
the header in the file says so), and the Burn engine and standard bootstrapper application inside
`OpenCraft-Studio-Setup-*.exe` are part of WiX under the same licence. The OpenCraft Studio mark
(four tiles) is drawn by `installer/build.ps1` and is original work, MIT OR Apache-2.0.
