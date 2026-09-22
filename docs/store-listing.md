# Store listing draft (M10/M11 prep)

תוכן מוכן-להדבקה לעמוד itch.io ולטופס החנות של Google Play, כדי לחסוך
למפתח את הצורך לכתוב את זה מהתחלה ברגע שהחשבונות (itch.io / Play Console
$25 חד-פעמי) קיימים — ראה docs/GDD.md §7.8's checklist על החלוקה
קוד-מול-משתמש. שום דבר כאן לא דורש חשבון כדי להכין; רק להדביק כשהחשבון
כבר קיים.

**נבדק מול docs/GDD.md §4's טבלת מותר/אסור** לפני הכתיבה: אין "The
Incredible Machine"/"TIM"/"Rube Goldberg" (סימן מסחר רשום) בשום מקום
כאן, אין השוואה מפורשת ("like TIM", "a clone of...") — בדיוק המלכודת
ש-§4.2 מזהירה מפניה במטא-דאטה של חנות. השתמשתי בתיאור הגנרי המפורש
שהמסמך עצמו מציע: "chain-reaction contraption puzzles".

---

## App name

**ChainWorks** — matches `Cargo.toml`'s `apk_label`/`package`
(`com.chainworksgame.app`) exactly; the store listing name should stay
consistent with what actually shows on the device/desktop.

## Short description (Play Store: 80 chars max)

> Build chain-reaction contraptions. Physics does the rest.

(59 characters — leaves headroom if Play trims differently than expected.)

## Long description

> Drop a ball, launch a spring, light a fuse — and watch one small push
> ripple into a whole contraption of gears, ropes, fans, and switches
> until it finally reaches the goal.
>
> ChainWorks is a chain-reaction physics puzzle game. Every level gives
> you a handful of real, physically-simulated parts — planks, pulleys,
> gears, conveyor belts, fuses, fans, switches — and one goal: get the
> ball (or the balloon, or the crate) safely where it needs to go. There's
> no single correct answer waiting to be found by trial and error; every
> part behaves like the real thing would, so once you understand *why*
> a lever tips or a gear train reverses direction, you can build your own
> solution from scratch.
>
> - **60 handcrafted levels** across four chapters, from a single new
>   part introduced gently to long causal chains combining a dozen
>   mechanics at once.
> - **Real physics, not scripted animation** — every collision, every
>   rope tension, every gear ratio is simulated, and it's deterministic:
>   solve a level once, and it solves the same way every time.
> - **A full sandbox mode** with every part unlocked and no level to
>   fail — save and load your own contraptions freely.
> - Energy flows through your machine in visible, color-coded lines —
>   rotary power, tension, electricity, heat, air pressure — so you can
>   actually see *why* something works, not just that it does.
>
> No ads. No in-app purchases. No account, no network connection needed
> at all — just you, some parts, and a problem to solve.

## Feature bullets (short form, for a store page's bullet list)

- 60 physics puzzle levels across 4 chapters
- Real, deterministic rigid-body physics simulation
- Full sandbox/free-build mode with save & load
- Color-coded energy flow (gears, ropes, belts, wires, heat, wind)
- No ads, no in-app purchases, no account required
- Desktop (Windows/macOS/Linux) and Android

## Genre / tags

Puzzle, Physics, Simulation, Sandbox, Casual, Singleplayer, 2D

(itch.io tags field — lowercase, comma-separated: `puzzle, physics,
simulation, sandbox, casual, singleplayer, 2d, indie`)

## Content rating questionnaire — factual answers

The actual game has none of the following, so every content-rating
questionnaire (Play Console, itch.io's own rating field) should answer
"no"/"none" straight down the list — this isn't a guess, it's what the
code actually does:

- Violence: none (cartoon physics — objects break/pop, no characters,
  no blood/gore).
- Sexual content: none.
- Profanity: none (no dialogue or text beyond level titles/goals).
- Simulated gambling: none.
- User-generated content shared with others: none — sandbox saves stay
  on the player's own device (`saves/` locally); there's no upload/share
  feature to any server.
- Ads: none — grep confirms no ad SDK anywhere in the dependency tree.
- In-app purchases: none — no monetization code exists at all.

This should land as the mildest possible rating on every platform (Play
Console: "Everyone" / PEGI 3 equivalent; itch.io: no content warnings
needed).

## Google Play "Data Safety" form — factual answers

Verified against the actual code, not assumed: `grep -rn "reqwest\|http::\|
TcpStream\|analytics\|telemetry\|UdpSocket" src/ Cargo.toml` returns
nothing — there is no networking code and no analytics/telemetry SDK
anywhere in this project's dependency tree. `Cargo.toml`'s
`[package.metadata.android]` also declares no permissions at all (no
`INTERNET`, no storage permission — Android's own app-private storage,
which `save_load.rs` writes to via `AndroidApp::internal_data_path()`,
needs no permission grant on modern Android).

So the Data Safety form should be answered:

- **Does your app collect or share any of the required user data
  types?** No.
- **Data collected:** none (no location, personal info, financial info,
  health, messages, photos/videos, audio, files/docs, app activity,
  device/other IDs — none of it is read or transmitted, because there is
  no code path that could).
- **Data shared with third parties:** N/A — none collected, none shared.
- **Is all user data encrypted in transit?** N/A — nothing is
  transmitted at all; the app makes no network requests.
- **Can users request data deletion?** N/A — nothing is collected to
  delete; a player's own save files live only in their own device
  storage under their own control (delete the file/uninstall the app).

## itch.io-specific fields

- **Kind of project:** Game
- **Release status:** Released (once M10/M11's actual blockers — see
  docs/GDD.md §7.8 — are cleared)
- **Pricing:** developer's own call, not something this doc should
  presume (free / name-your-price / fixed price are all viable for a
  puzzle game like this)
- **Platforms:** Windows, macOS, Linux (matches `desktop.yml`'s/
  `release.yml`'s build matrix exactly — don't list a platform that
  isn't actually built and tested in CI)

## What this doc deliberately does not decide

- **Pricing.** A business decision, not implied anywhere above.
- **Screenshots/trailer/icon artwork for the store page itself.** This
  doc is copy, not assets — the actual capture/export of screenshots
  should come from a real run once the visual-polish work in
  docs/GDD.md §3.9 is further along, so the store page reflects what
  players will actually see.
- **The Android package name.** `com.chainworksgame.app` is explicitly
  a placeholder per `Cargo.toml`'s own comment and docs/GDD.md §7.8 —
  changing it is a one-way decision only the developer can make (tied to
  their own domain/org name), and it must happen *before* any real Play
  Console submission, not after.
