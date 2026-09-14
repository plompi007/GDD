# GDD — "CHAINWORKS" (שם עבודה)

מסמך מפרט פיתוח מלא — משחק פאזל פיזיקלי מבוסס תגובת שרשרת

**גרסה:** 1.3 | **תאריך:** ספטמבר 2026
**קהל היעד של המסמך:** סוכן קוד (Claude Code) + המפתח
**פלטפורמות יעד:** דסקטופ (Windows/Mac/Linux) **ואנדרואיד** מוקדם מאוד באותו קוד (Bevy תומך בזה natively) — ראה סעיף 7.5. iOS/Web נשארים בהמשך.

> **הערה משפטית מקדימה:** המסמך הזה מנתח מכניקות משחק (שאינן מוגנות בזכויות יוצרים) ומגדיר יצירה מקורית חדשה. אין להשתמש בשם, בגרפיקה, בצלילים, בשמות הדמויות או בפריסות השלבים של אף משחק קיים. ראה סעיף 4.

> **מה חדש בגרסה 1.1:** (א) חיזוק מפורש של עקרון "**מקסימום נאמנות מכנית/חווייתית ל-TIM, מינימום חפיפה משפטית**" — סעיף 1.1 ו-4.1. (ב) זהות חזותית חדשה — כיוון אמנותי פרימיום ברמת 2026, מחליף את "Timber & Brass" כהמלצה ראשית (הכיוונים הקודמים נשמרו כחלופות מתועדות) — סעיף 4.3. (ג) שפת תנועה, Juice ומערכת עיצוב (design tokens) ברמת גימור של משחק מודרני — סעיף 3.9 החדש.

> **מה חדש בגרסה 1.2 — פיבוט סטאק ל-Rust:** לאחר סקירה ישירה של [OpenTIM](https://github.com/mrfixit2001/OpenTIM) (רה-implementation קוד-פתוח, GPL-3.0, של The Even More! Incredible Machine), הוחלט לעבור מ-TypeScript/Pixi/Capacitor ל-**Rust + nannou + rapier2d, דסקטופ-קודם** — מראה ארכיטקטורת המודולים של OpenTIM עצמה (`part.rs`, `level_file_format.rs`, `atmosphere.rs`) קרובה מאוד למה שהמסמך הזה כבר ביקש (חלוקה ל-parts/level/sim), ורספייר הוא ממילא crate-Rust מקורי — הבינדינגים ל-JS/WASM שהיו בתכנון v1.1 היו שכבת עטיפה מיותרת. **חשוב: זו השראה ארכיטקטונית בלבד — אין תלות או fork בקוד ה-GPL-3.0 של OpenTIM עצמו**, כדי ש-ChainWorks יישאר קניין רוחני עצמאי (ראה §2.1). סעיפים 1, 3, 4 (מכניקה, UX/זהות חזותית, משפטי) נשארים תקפים במלואם — הם בלתי תלויים בשפת המימוש.

> **מה חדש בגרסה 1.3 — פיבוט מנוע שני: nannou → Bevy:** המפתח משחק מהאנדרואיד שלו. `nannou` (v1.2) לא תומך במובייל כלל — היה דוחה את האפשרות לשחק בפועל לשלב מאוחר מאוד (M12). **Bevy** הוא מנוע Rust בשל שתומך Android/iOS/Desktop/Web **מאותו קוד** מהיום הראשון, עדיין MIT/Apache-2.0 (אין GPL), ועדיין שומר על אותה ארכיטקטורת מודולים בהשראת OpenTIM — רק דרך ECS (Entities/Components/Systems) של Bevy במקום struct בודד עם update/view. `rapier2d` נשאר, דרך הפלאגין הרשמי `bevy_rapier2d`. **⚠️ הערת גרסאות קריטית:** Bevy 0.19.x (העדכני ביותר בזמן הכתיבה) שבור כרגע מול glam 0.32.1/serde — `bevy_reflect` לא מתקמפל (E0277 על `BVec3A`/`BVec4A`). **מוצמד בכוונה ל-Bevy 0.16 + bevy_rapier2d 0.30** (הזוג היציב האחרון לפני ה-regression, מאומת שמתקמפל ורץ). ראה §2.1a לפירוט המלא ולסימנים לבדוק לפני שמנסים לשדרג בעתיד.

---

## 0. TL;DR — ההחלטות הטכניות המרכזיות

| החלטה | הבחירה | למה |
|---|---|---|
| סטאק | **Rust + `bevy` 0.16 (ECS/רינדור/חלון) + `bevy_rapier2d` 0.30 (פיזיקה, `enhanced-determinism`) + `serde`/`serde_json`** | דטרמיניזם מובטח מהקופסה (rapier2d הוא Rust מקורי), Bevy תומך Android/iOS/Desktop/Web מאותו קוד, ועדיין שומר על ארכיטקטורת מודולים בהשראת OpenTIM (`part.rs`, `level_file_format.rs`, `atmosphere.rs`) — עכשיו כ-ECS Components/Systems — בלי תלות בקוד ה-GPL-3.0 שלהם |
| מנוע פיזיקה | `bevy_rapier2d` (עוטף `rapier2d` native Rust, feature `enhanced-determinism`) | דטרמיניזם cross-platform מובטח + snapshot/restore מובנה, בלי overhead של WASM. קריטי לפאזל שבו פתרון חייב להיות ניתן לשחזור |
| ארכיטקטורת ליבה | סימולציה דו-שכבתית: שכבת גופים קשיחים (rapier2d, כ-Bevy Components) + שכבת גרף אנרגיה לוגי מעליה (Bevy Systems/Resources) | זה הסוד של הז'אנר. חבלים, גלגלי שיניים, חשמל וחום אינם פיזיקה קשיחה — הם רשת סיגנלים. ניסיון לסמלץ חבל כשרשרת גופים = חוסר יציבות ובאגים אינסופיים |
| Timestep | קבוע 1/120s (`TimestepMode::Fixed`), מקסימום 4 substeps | דטרמיניזם + יציבות מפרקים |
| נתוני שלבים | JSON חיצוני + JSON Schema + `serde` (deserialize + ולידציה ידנית/`jsonschema` crate) | שלבים ניתנים לעריכה בלי לגעת בקוד; Claude Code יכול לייצר שלבים כ-data |
| **זהות חזותית (v1.1)** | **Prism Foundry — "מעבדה קינטית" פרימיום, וקטור שטוח + עומק אור רך, קידוד צבע לפי סוג אנרגיה** | נראה כמו משחק 2D מקורי שיצא ב-2026 — לא נוסטלגי, לא "משחק אינדי בסיסי". ניטרלי משפטית לחלוטין (אינו מבוסס על שום נכס קיים) |
| **פלטפורמה (v1.3)** | **דסקטופ + אנדרואיד מוקדם, מאותו קוד Bevy** | המפתח משחק מהאנדרואיד — צריך build שרץ על הטלפון בפועל תוך כמה milestones, לא בסוף. ראה §7.5 |

**חלופה** שנשקלה ונדחתה: הישענות ישירה על ה-crate של OpenTIM עצמו. **נפסלה** — OpenTIM הוא GPL-3.0, ותלות/fork בו הייתה מחייבת את ChainWorks לצאת גם הוא GPL-3.0 (קוד פתוח לגמרי, בלי אפשרות למוצר סגור/בתשלום). הפתרון: מראים את הצורה הארכיטקטונית, כותבים קוד מקורי. ראה §2.1.

**חלופה שנייה שנשקלה ונדחתה (v1.2 → v1.3):** `nannou`. תוצאה נהדרת לדסקטופ בלבד, אבל **לא תומך במובייל בכלל** (בנוי על `winit`+`wgpu` בקונפיגורציית דסקטופ קשיחה) — היה דוחה משחק בפועל על הטלפון לשלב אחרון מאוד. Bevy נבחר במקומו כי הוא תומך Android מהיום הראשון, עדיין Rust, עדיין MIT/Apache-2.0.

---

## 1. ניתוח רכיבים ומכניקות (Core Gameplay Mechanics)

### 1.1 ניתוח הז'אנר — מה באמת גורם לו לעבוד

לפני רשימת הרכיבים, חמישה עקרונות שהם הליבה האמיתית של הז'אנר. כל החלטת עיצוב במסמך נגזרת מהם:

1. **דטרמיניזם מוחלט.** אותה הצבה = אותה תוצאה, תמיד. בלי זה הפאזל נהפך להימור, והשחקן מאבד אמון.
2. **צפיפות סיבתית.** כל רכיב חייב להשפיע על לפחות 3 רכיבים אחרים בדרכים שונות. רכיב שעושה דבר אחד = רכיב משעמם.
3. **מחסור מכוון.** ארגז החלקים מוגבל. הכיף הוא לא "מה לשים" אלא "איך לעשות עם מה שיש".
4. **פתרונות מרובים.** תנאי הניצחון בודק תוצאה, לא שיטה. אף פעם אל תבדוק "האם השחקן שם גלגלת במיקום X".
5. **טיימינג כמימד שלישי.** שני פתרונות זהים מרחבית יכולים להיכשל/להצליח לפי סדר האירועים. זה מה שנותן עומק אחרי 50 שלבים.

> #### עיקרון-על (חדש ב-1.1): מקסימום נאמנות, אפס חפיפה
>
> **חוקי משחק, יחסי פיזיקה, וארכיטיפים תפקודיים של רכיבים אינם מוגנים בזכויות יוצרים — רק הביטוי הקונקרטי שלהם (שם, גרפיקה, צליל, פריסת שלב, זהות דמות) מוגן.** המשמעות המעשית לסוכן הקוד:
>
> - **בדיוק המכניקה — ליברלי לגמרי.** אם ב-TIM כדור עופרת שוקל יותר ומאיץ מהר יותר מכדור גומי, ל-`ball_lead` ול-`ball_rubber` כאן צריכים להתנהג **באותה מידה יחסית** (יחסי מסה, restitution, friction דומים ככל שהפיזיקה מאפשרת). אם גלגלת הופכת כיוון משיכה ביחס 1:1 — כך גם כאן. השאיפה היא ש**שחקן שמכיר את TIM ירגיש "אני יודע איך זה עובד" מהשנייה הראשונה**, כי היחסים הפיזיקליים והתפקודיים זהים.
> - **צפיפות התוכן — ליברלי לגמרי.** מספר סוגי הרכיבים (52), עומק מטריצת האינטראקציות (סעיף 1.4), ומורכבות הקמפיין (60 שלבים ב-4 חטיבות) נועדו להיות **לפחות ברמת** העומק המכני של The Even More! Incredible Machine, לא פחות ממנו. אל תצמצם רכיב רק כי "מספיק דומה" — אם יש מנגנון ב-TIM שאין לו מקבילה כאן, זו באג בכיסוי, לא עודף.
> - **הביטוי החזותי/שמי — אפס חפיפה, בלי פשרות.** שם, ספרייט, פלטת צבעים, עיצוב דמות, צליל, מוזיקה, טקסט שיווקי, ופריסת שלב חייבים להיות **מקוריים ב-100%** ולא "בהשראת" באופן מזוהה. ראה טבלת השמות בסעיף 4.4 — זו לא רשימת המלצות, זו רשימת חובה.
> - מבחן פשוט לכל תוספת עתידית: *"אם אני מסתיר את השם ואת הגרפיקה, האם מהנדס פיזיקה שמכיר את שני המשחקים היה מזהה שזו אותה מכניקה?"* — **כן זו התשובה הרצויה.** *"האם צופה מזדמן שמסתכל על המסך היה חושב שזה אותו משחק?"* — **לא, לעולם לא.**

### 1.2 טקסונומיית אנרגיה — הבסיס לכל המערכת

כל רכיב מוגדר כ-צומת בגרף עם פורטים (in/out) מטיפוסים הבאים. זו האבסטרקציה שהקוד כולו נבנה סביבה:

```ts
export enum EnergyType {
  ROTARY    = 'ROTARY',    // סיבובי (מומנט) rad/s + direction
  TENSION   = 'TENSION',   // משיכה ליניארית (חבל) — עוצמה + כיוון
  ELECTRIC  = 'ELECTRIC',  // בוליאני on/off (+ לעומס זרם)
  THERMAL   = 'THERMAL',   // חום/להבה — יש מיקום ורדיוס
  PNEUMATIC = 'PNEUMATIC', // זרם אוויר — וקטור + עוצמה + טווח
  LIGHT     = 'LIGHT',     // חרוט/קרן — ray-cast
  IMPACT    = 'IMPACT',    // אירוע מגע חד-פעמי (טריגר)
}
```

**כלל ברזל לסוכן הקוד:** רכיב לא "יודע" על רכיב אחר. הוא רק פולט/צורך טיפוס אנרגיה. כל אינטראקציה חדשה = צירוף חדש של פורטים, בלי `if (partA === 'candle' && partB === 'fuse')`.

### 1.3 קטלוג הרכיבים המלא

עמודות: `Tier` = P0 (MVP), P1, P2. `mass` בק"ג, `rest` = restitution, `fric` = friction, `gScale` = gravityScale.

#### א. מבנה וסטטיקה (STATIC — לא מושפעים מכבידה)

| ID | שם תצוגה | תכונות | פורטים | Tier |
|---|---|---|---|---|
| `plank_wood` | קרש עץ | static, סיבוב חופשי, fric 0.55, rest 0.20 | — | P0 |
| `beam_steel` | קורת פלדה | static, fric 0.25, rest 0.35, tag: METALLIC | — | P0 |
| `wall_brick` | קיר לבנים | static, fric 0.70, rest 0.05, tag: DESTRUCTIBLE | — | P0 |
| `ramp_arc` | מסלול מעוגל | static, fric 0.15 | — | P1 |
| `pipe_straight` / `pipe_elbow` | צינור / ברך | static, סנסור פנימי שמכוון גוף לאורך ה-spline, מנטרל כבידה בפנים | — | P1 |
| `spike_pin` | יתד | static זעיר, tag: SHARP (מפוצץ בלונים) | — | P0 |
| `floor_ground` | רצפה | static, גבול העולם | — | P0 |

#### ב. גופים ניידים (DYNAMIC)

| ID | שם תצוגה | mass | rest | fric | gScale | tags |
|---|---|---|---|---|---|---|
| `ball_lead` | כדור עופרת | 8.0 | 0.05 | 0.45 | 1.0 | HEAVY, METALLIC |
| `ball_iron` | כדור ברזל | 5.0 | 0.15 | 0.35 | 1.0 | METALLIC |
| `ball_wood` | כדור עץ | 1.2 | 0.40 | 0.50 | 1.0 | FLAMMABLE |
| `ball_rubber` | כדור גומי | 0.6 | 0.85 | 0.80 | 1.0 | BOUNCY |
| `ball_glass` | גולת זכוכית | 0.30 | 0.60 | 0.05 | 1.0 | FRAGILE |
| `crate_wood` | ארגז | 3.0 | 0.10 | 0.65 | 1.0 | FLAMMABLE, DESTRUCTIBLE |
| `balloon_lift` | בלון | 0.15 | 0.50 | 0.10 | −0.35 | FLAMMABLE, POPPABLE, WIND |
| `drifter_orb` | כדור אתר | 1.0 | 0.90 | 0.02 | 0.0 | linearDamping 0.15, מכפיל רגישות WIND×3.0 |

> **הערה לסוכן הקוד:** `gScale` שלילי לבלון מיושם כ-`rigidBody.setGravityScale(-0.35)`. לא כוח מותאם. לשדות אוויר, מאוחסן ב-metadata של הרכיב ומופעל ע"י `FieldSystem`.

#### ג. העברת כוח (Mechanism — שכבת הגרף)

| ID | שם תצוגה | תיאור מכני | פורטים | Tier |
|---|---|---|---|---|
| `rope` | חבל | קו בין 2 עוגנים. **לא גוף פיזיקלי.** מעביר TENSION. אורך מקסימלי. נחתך ע"י SHARP או THERMAL | 2× TENSION (דו-כיווני) | P0 |
| `pulley_wheel` | גלגלת | מפנה כיוון של חבל, משנה יחס כוח 1:1. ניתן להצמדה לקיר או לגוף | 2× TENSION | P0 |
| `gear_small` / `gear_large` | גלגל שיניים | שני גלגלים ROTARY. צומת סמוכים (מרחק > r₁+r₂+ε) → צימוד עם היפוך כיוון ויחס r₁/r₂ | ROTARY in/out | P0 |
| `drive_belt` | רצועת הנעה | מחבר שני צמתי ROTARY מרוחקים, **ללא** היפוך כיוון. אורך מקסימלי 400 יח' | 2× ROTARY | P0 |
| `lever_seesaw` | נדנדה / מנוף | RevoluteJoint סביב ציר. עוגני חבל בשני הקצוות. ממיר תנועה↔TENSION וסיבוב↔תנועה | 2× TENSION anchors | P0 |
| `conveyor` | מסוע | ROTARY in → surfaceVelocity על הקוליידר. גופים מעל נעים לכיוון א', גופים מתחת לכיוון ב' | ROTARY in | P1 |
| `winch_drum` | תוף כננת | ROTARY in → TENSION out (מגלגל חבל) | ROTARY in, TENSION out | P1 |
| `axle_platform` | פלטפורמה על ציר | גוף דינמי מעוגן בציר, נוטה לפי משקל | — | P1 |

#### ד. מקורות אנרגיה

| ID | שם תצוגה | תיאור | פורטים | Tier |
|---|---|---|---|---|
| `outlet_power` | שקע חשמל | מקור ELECTRIC קבוע. פרמטר `startsOn: bool` | ELECTRIC out ×N | P0 |
| `motor_electric` | מנוע | ELECTRIC in → ROTARY out. פרמטרים: `rpm` (1–600), `direction` | ELECTRIC in, ROTARY out | P0 |
| `switch_plate` | לוח לחיצה | IMPACT in → toggle/pulse של ELECTRIC out. פרמטר `mode: 'toggle'|'momentary'` | IMPACT in, ELECTRIC out | P0 |
| `flywheel_spring` | גלגל תנופה קפיצי | נדרך ידנית בעורך (`charge`: 1–5), משחרר ROTARY לזמן קצוב בהפעלת IMPACT | IMPACT in, ROTARY out | P1 |
| `wind_vane` | כנף רוח | PNEUMATIC in → ROTARY out | PNEUMATIC in, ROTARY out | P1 |
| `turbine_steam` | טורבינת קיטור | PNEUMATIC in (קיטור חזק בלבד) → ROTARY out | PNEUMATIC in, ROTARY out | P2 |
| `cell_solar` | תא סולארי | LIGHT in → ELECTRIC out | LIGHT in, ELECTRIC out | P1 |

#### ה. פנאומטיקה ואוויר

| ID | שם תצוגה | תיאור | פורטים | Tier |
|---|---|---|---|---|
| `fan_blower` | מאוורר | ELECTRIC in → שדה מלבני PNEUMATIC. פרמטרים: `power` 1–3, `range` | ELECTRIC in, PNEUMATIC out | P0 |
| `bellows` | מפוח | לחיצה (IMPACT מלמעלה/מלמטה) → פולס אוויר חד-פעמי | IMPACT in, PNEUMATIC out | P0 |
| `nozzle_vacuum` | פיית שאיבה | ELECTRIC in → שדה משיכה (PNEUMATIC שלילי) | ELECTRIC in, PNEUMATIC out | P1 |
| `kettle_steam` | קומקום | THERMAL in → סילון קיטור עוצמתי, PNEUMATIC מכוון (טווח קצר) | THERMAL in, PNEUMATIC out | P1 |

#### ו. חום ופירוטכניקה

| ID | שם תצוגה | תיאור | פורטים | Tier |
|---|---|---|---|---|
| `candle` | נר | פולט THERMAL ברדיוס 20 יח'. נכבה מ-PNEUMATIC חזק. `startsLit: bool` | THERMAL out | P0 |
| `burner_torch` | מבער | ELECTRIC in → THERMAL out (להבה מכוונת, טווח 60) | ELECTRIC in, THERMAL out | P1 |
| `fuse_cord` | פתיל | THERMAL in בקצה א' → התקדמות בעירה 45 יח'/שנייה → THERMAL out בקצה ב'. אלמנט הטיימינג המרכזי במשחק | THERMAL in, THERMAL out | P0 |
| `charge_barrel` | מטען נפץ | THERMAL in → אימפולס רדיאלי (עוצמה `power`, רדיוס `radius`) + הריסת DESTRUCTIBLE בטווח | THERMAL in, IMPACT out | P0 |
| `lens_magnifier` | עדשה מגדלת | LIGHT in → THERMAL out בנקודת המוקד | LIGHT in, THERMAL out | P2 |

#### ז. אור

| ID | שם תצוגה | תיאור | פורטים | Tier |
|---|---|---|---|---|
| `lamp_bulb` | נורה | ELECTRIC in → חרוט LIGHT | ELECTRIC in, LIGHT out | P1 |
| `beam_emitter` | פנס קרן | ELECTRIC in → קרן LIGHT ישרה (ray-cast) | ELECTRIC in, LIGHT out | P1 |
| `mirror_panel` | מראה | מחזירה קרן LIGHT בזווית. סיבוב בקפיצות 15° | LIGHT passthrough | P2 |
| `sensor_photo` | חיישן אור | LIGHT in → ELECTRIC out | LIGHT in, ELECTRIC out | P1 |

#### ח. מפעילים קינטיים

| ID | שם תצוגה | תיאור | פורטים | Tier |
|---|---|---|---|---|
| `punch_arm` | זרוע הלימה | IMPACT על הכפתור בגב → אימפולס קדימה (עוצמה קבועה 22 N·s) | IMPACT in/out | P0 |
| `springboard` | קרש קפיצה | משטח עם restitution אפקטיבי 1.35 (שומר מומנטום אופקי) | — | P0 |
| `bumper_post` | פגוש | אימפולס רדיאלי בכל מגע. cooldown 100ms | IMPACT in/out | P1 |
| `tube_launcher` | צינור שיגור | מכיל גוף אחד. IMPACT/ELECTRIC → משגר בזווית ובעוצמה `angle`, `power` | in: IMPACT\|ELECTRIC | P1 |
| `spring_crate` | ארגז קפיצי | ROTARY in — אחרי N סיבובים הבמה נפתחת ומשגרת אימפולס. מאפשר השהיה מדויקת | ROTARY in, IMPACT out | P1 |
| `cutter_shears` | מספריים | IMPACT/ELECTRIC → חותך `rope` וגם POPPABLE בטווח | in: IMPACT\|ELECTRIC | P0 |
| `trapdoor` | דלת מלכודת | ELECTRIC in → הקוליידר הופך ל-sensor (נפתח) | ELECTRIC in | P1 |
| `magnet_electro` | אלקטרומגנט | ELECTRIC in → שדה משיכה על METALLIC בלבד | ELECTRIC in | P2 |
| `pad_antigravity` | לוח אנטי-כבידה | אזור שהופך `gravityScale` של כל גוף בתוכו | — | P2 |

#### ט. דמויות, יעדים ומיוחדים

| ID | שם תצוגה | תיאור | Tier |
|---|---|---|---|
| `walker_unit` | יחידת הליכה ("ווקר") | בוט קטן שצועד בכיוון קבוע, מסתובב כשנתקל במכשול, נופל עם כבידה. מוגן: אם `speed` < 14 בפגיעה → FAILED | P1 |
| `walker_beacon` | משואת משיכה | אם ה-ווקר "רואה" אותה בקו ראייה אופקי פנוי — הוא צועד לעברה | P1 |
| `bin_target` | דלי יעד | Sensor. נחשב "הכיל" אחרי שהגוף בתוכו ומהירותו > 0.5 למשך 500ms | P0 |
| `goal_zone` | אזור יעד | Sensor בלתי נראה לתנאי ניצחון גנריים | P0 |
| `shelter_walker` | מחסה | ה-ווקר נכנס ונעלם → תנאי ניצחון | P1 |

**סה"כ: ~52 רכיבים.** רק 24 מהם = P0 ומספיקים בהחלט ל-40 שלבים איכותיים.

### 1.4 מטריצת אינטראקציות (Interaction Matrix)

הכלל: מקור → יעד ⇒ תוצאה. זו הטבלה שהסוכן צריך לממש ב-`part.rs` (או מודול ייעודי `interaction_rules.rs`).

#### 1.4.1 אינטראקציות לפי טיפוס אנרגיה

| מקור ↓ / יעד → | TENSION | ROTARY | ELECTRIC | THERMAL |
|---|---|---|---|---|
| **TENSION** | מועבר דרך `pulley` (1:1, שינוי כיוון) | `winch_drum`: הפוך — משיכה → סיבוב | — | — |
| **ROTARY** | `winch_drum`: סיבוב → משיכה | `gear`: היפוך + יחס רדיוסים. `belt`: ללא היפוך | `generator` (P2) | — |
| **ELECTRIC** | — | `motor` → ROTARY | חיווט מקבילי דרך `outlet` | `burner_torch` → להבה |
| **THERMAL** | `rope` שורף → ניתוק | — | — | `fuse_cord` → התקדמות 45 יח'/s |
| **PNEUMATIC** | — | `vane_wind` → ROTARY | — | `candle` מכבה (power ≥ 2) |
| **LIGHT** | — | — | `sensor_photo`, `cell_solar` | `lens_magnifier` → מוקד |
| **IMPACT** | — | `flywheel_spring` | `switch_plate` | — |

#### 1.4.2 אינטראקציות לפי תגית (Tag Rules)

| תנאי | תוצאה |
|---|---|
| `SHARP` נוגע ב-`POPPABLE` | הבלון מתפוצץ → נפלט פולס קטן PNEUMATIC רדיאלי |
| `THERMAL` בטווח `FLAMMABLE` | הגוף נשרף ונעלם אחרי 800ms (נר על ארגז, בלון, חבל) |
| `THERMAL` בטווח `POPPABLE` | פיצוץ מיידי |
| `charge_barrel` מתפוצץ ליד `DESTRUCTIBLE` | הקוליידר מוסר מהעולם |
| `magnet_electro` פעיל + `METALLIC` בטווח | כוח `F = k·m / d²`, clamp ל-`d_min = 20` |
| `FRAGILE` בפגיעה ב-`speed` > 18 | הגוף מתנפץ ונעלם |
| `walker_unit` פוגע ב-`speed` > 14 | `FAILED` מיידי |
| `antigravity_pad` מכיל גוף | `gravityScale *= -1` בכניסה, חזרה ביציאה |

#### 1.4.3 שאלות ספציפיות שביקשת — התשובות המימושיות

**מה קורה כשחבל מתחבר לגלגלת?**

גלגלת היא נקודת ניתוב (`RopeNetwork`). החבל **אינו** גוף פיזיקלי. הוא קטע ברשת מתיחה שמפצלת את החבל לשני מקטעים ומשנה את כיוון וקטור הכוח. אלגוריתם בכל tick:

1. עבור כל רשת חבל: חשב `totalLength` = Σ |segment_i|
2. אם `totalLength > maxLength`: הרשת מתוחה (taut)
3. אם מתוחה: חשב את התזוזה הנדרשת `delta = totalLength - maxLength`
4. פזר את `delta` בין שני העוגנים ביחס הפוך למסה האפקטיבית שלהם
5. הפעל אימפולס על כל עוגן בכיוון המקטע הצמוד אליו (לא הישר בין הקצוות!)
6. אם עוגן הוא `static` → כל התזוזה נופלת על העוגן השני

זה נותן את ההתנהגות "חבל שמושך" בלי חוסר-היציבות של שרשרת מפרקים.

**מה קורה כשלהבה נוגעת בפתיל?**

`ThermalSystem` מבצע בכל tick שאילתת טווח (`world.intersectionsWithShape`) סביב כל פולט THERMAL. פתיל שנכנס לטווח משנה `state: 'IDLE' → 'BURNING'` ומתחיל `burnProgress += 45 * dt`. בכל פריים נקודת הבעירה עצמה נהפכת לפולט THERMAL זמני ברדיוס 8 — כך שפתיל יכול להצית פתיל אחר, בלון או ארגז שנמצאים לאורכו. כש-`burnProgress ≥ length` נפלט THERMAL בקצה השני והפתיל נעלם.

**מה קורה כשהדף אוויר מזיז מניפה?**

`FieldSystem` מחזיק רשימת שדות פעילים. כל `tick`:

```
for field in activeFields:
  bodies = world.intersectionsWithAABB(field.aabb)
  for body in bodies:
    f = field.direction * field.power * body.windFactor * falloff(dist)
    body.applyForce(f)
    if body.partType has PNEUMATIC-in port:
      graph.emit(body.nodeId, PNEUMATIC, f.magnitude)
```

כלומר: אותו שדה גם דוחף גופים פיזית וגם מזין את הפורט הלוגי. `wind_vane` ממיר את זה ל-`angularVelocity = clamp(power * 0.8, 0, 12) rad/s` ומזרים החוצה ROTARY לגרף.

---

## 2. מנוע המשחק והפיזיקה (Physics & Logic Blueprint)

### 2.1 בחירת מנוע — נימוק (מעודכן v1.2)

| מנוע | דטרמיניזם חוצה-פלטפורמות | יציבות מפרקים | ביצועים | הכרעה |
|---|---|---|---|---|
| **`rapier2d` (Rust, native)** | ✅ מובטח (feature `enhanced-determinism`) + snapshot/restore מובנה | טובה מאוד | הכי מהיר (native, אין WASM) | ✅ **נבחר** |
| Box2D (בינדינג Rust) | ⚠️ תלוי בבינדינג, לא כולם שומרים על IEEE-754 קפדני | הטובה ביותר | טובה | חלופה סבירה |
| מימוש פיזיקה עצמי (כמו OpenTIM) | ✅ מובטח מטבעו (הקוד שלך) | תלוי כמה זמן משקיעים | לרוב פחות יציב מ-rapier | נשקל ונדחה — ראה מטה |

**למה לא ללכת בדיוק כמו OpenTIM ולממש פיזיקה משלנו?** OpenTIM שם לעצמו מטרה שונה מהותית: **לשחזר בדיוק ביט-לביט את אלגוריתם הפיזיקה המקורי של Windows 3.1 מ-1994, כולל הבאגים שלו** — זו הסיבה שיש להם `reverse-engineering/` עם ניתוח Ghidra של ה-binary המקורי. ChainWorks הוא **משחק חדש** שרק שואף ליחסי-כוח *דומים* (סעיף 1.1), לא לשחזור בייט-מדויק של מנוע ישן — אז אין סיבה לוותר על מנוע פיזיקה מודרני, יציב ומהיר יותר. `rapier2d` הוא הבחירה הנכונה כאן בדיוק כמו ב-v1.1, רק שעכשיו — בהיותנו ב-Rust — משתמשים בו **ישירות כ-crate מקורי**, בלי שכבת עטיפת WASM/JS-bindings (`@dimforge/rapier2d-deterministic` ב-npm) שהייתה נחוצה רק בגלל שהסטאק הקודם היה TypeScript.

**הנימוק המכריע לדטרמיניזם:** snapshot/restore מובנה (`RigidBodySet`/`ColliderSet` הם `Clone`, וניתן לבצע `bincode`/`serde` serialize מלא של מצב הסימולציה) מאפשר `golden replay tests` — לשמור פתרון כ-JSON, להריץ אותו ב-CI (`cargo test`), ולוודא שהשלב עדיין פתיר אחרי כל שינוי קוד. בלי זה, כל tweak לפיזיקה שובר שלבים בשקט.

**חבילה:** `rapier2d` עם `features = ["enhanced-determinism"]` (**לא** `simd8` — שני ה-features סותרים זה את זה, ראה אזהרת קומפילציה בקוד המקור של rapier).

**⚠️ אין תלות בקוד של OpenTIM עצמו.** OpenTIM הוא GPL-3.0 — לו ChainWorks תלוי או fork-י בקוד שלו, ChainWorks היה חייב לצאת גם הוא GPL-3.0 (קוד פתוח לגמרי, בלי אפשרות למוצר סגור/מכירה בחנויות בלי לפרסם את כל קוד המקור). ההחלטה: ללמוד מהצורה הארכיטקטונית שלהם (חלוקת מודולים — ראה §5.5) ולכתוב קוד מקורי משלנו מאפס. זה בדיוק כמו העיקרון בסעיף 1.1 לגבי מכניקות משחק — הרעיון/המבנה מותר להשראה, המימוש הקונקרטי חייב להיות מקורי.

### 2.1a מנוע רינדור/חלון — nannou → Bevy (v1.3)

`rapier2d` (§2.1) נשאר ללא שינוי מ-v1.2. מה שהתחלף הוא שכבת החלון/רינדור/ECS שעוטפת אותו:

| מנוע | תמיכת מובייל | ECS/ארגון קוד | רישוי | הכרעה |
|---|---|---|---|---|
| **Bevy** | ✅ Android + iOS + Desktop + Web מאותו קוד | ECS מלא (Entities/Components/Systems) | MIT/Apache-2.0 | ✅ **נבחר (v1.3)** |
| `nannou` | ❌ דסקטופ בלבד (`winit`+`wgpu` בקונפיגורציה קשיחה) | Model/update/view (לא ECS) | MIT/Apache-2.0 | היה נבחר ב-v1.2, נדחה |
| מימוש חלון/רינדור עצמי (`winit`+`wgpu` ישירות) | ✅ (winit תומך מובייל) | חופשי, אבל את הכול כותבים ידנית | — | יותר מדי עבודה ידנית מול Bevy הבשל |

**הנימוק:** המפתח משחק מהטלפון האנדרואיד שלו. עם `nannou`, "לשחק במשחק" היה נדחה לשלב M12 (אחרי launch דסקטופ מלא) כי `nannou` פשוט לא יודע לרוץ על אנדרואיד — צריך היה להחליף מנוע בכל מקרה מתישהו. Bevy עושה את זה **מהיום הראשון** מאותו קוד, בלי תאריך יעד רחוק. המחיר: Bevy הוא מסגרת ECS גדולה יותר מ-nannou (יותר concepts ללמוד: Components, Systems, Resources, Schedules, Plugins) — אבל זה בדיוק גם היתרון: ECS ממפה טבעי מאוד למודל "רכיב = צומת בגרף עם רכיבים (components) ומערכות (systems) שקוראות אותם" שכל המסמך הזה כבר מתאר (סעיף 1.2), ומחליף בצורה נקייה יותר את ה-trait `PartBehavior` עם ה-virtual dispatch שלו (ראה §5.6 המעודכן).

**איך זה משנה את הארכיטקטורה (§5.5):** מבנה הקבצים (`part.rs`, `level_file_format.rs`, `atmosphere.rs` וכו') נשאר כמעט זהה — זו עדיין השראה מ-OpenTIM. מה שמשתנה זה **מה שבתוך כל קובץ**: strcuts הופכים ל-`#[derive(Component)]`, ולוגיקה שהייתה מתודה על struct (`on_tick`, `on_energy`) הופכת ל-Bevy System שרץ על query של entities עם הרכיבים הרלוונטיים. זה טבעי יותר ל-ECS ופחות boilerplate מ-trait objects.

> ### ⚠️ הערת גרסאות קריטית — אל תשדרג את Bevy בלי לבדוק את זה קודם
>
> **מוצמד בכוונה:** `bevy = "0.16"` + `bevy_rapier2d = "0.30"`. **לא** הגרסה העדכנית ביותר (0.19.x / 0.36.x בזמן הכתיבה).
>
> **הסיבה:** `bevy_reflect` בגרסאות 0.19.x דורש `glam >= 0.32`, וב-`glam 0.32.1` המימושים של `Serialize`/`Deserialize` עבור `BVec3A`/`BVec4A` כתובים מול ה-trait של `serde_core` באופן שלא מספק את ה-bound שדרוש (`E0277: the trait bound BVec3A: serde::Serialize is not satisfied`) — קורה עם `bevy = "0.19"` בכל קונפיגורציית features שניסינו (כולל `bevy_rapier2d` עם `default-features = false`). זו לא באגיה שלנו לתקן — זה regression ב-ecosystem (כנראה קשור למעבר של serde ל-`serde_core` split) שממתין לתיקון upstream.
>
> **לפני שמנסים לשדרג בעתיד:** ודא ש-`cargo build` נקי לפני שמשנים גרסה, ואם נתקלים שוב ב-E0277 על `glam::BVec3A`/`BVec4A` בתוך `bevy_reflect` — זו אותה בעיה. בדוק את `bevy_reflect`'s `Cargo.toml` (`[dependencies.glam]`) בגרסה שאתה שוקל: אם `glam >= 0.32`, ייתכן שהבאג עדיין קיים; חפש גרסת `bevy`/`bevy_rapier2d` תואמת שבה `bevy_reflect` עדיין תלוי ב-`glam < 0.32` (למשל, לבדוק את `bevy_reflect` בהתקנה המקומית: `find ~/.cargo/registry/src -iname "bevy_reflect-*"` ואז `grep glam Cargo.toml`).

### 2.2 קנה מידה ופרמטרים גלובליים

```rust
// core/sim/constants.rs
pub const PIXELS_PER_METER: f32 = 32.0; // rapier2d עובד במטרים. לעולם אל תשלח פיקסלים.
pub const GRAVITY_Y: f32 = -9.81;       // ⚠️ nannou הוא +y-up (סטנדרט מתמטי/OpenGL),
                                         //    בשונה מ-v1.1 (Pixi, +y-down). ראה הערה מתחת.
pub const FIXED_DT: f32 = 1.0 / 120.0;
pub const MAX_SUBSTEPS_PER_FRAME: u32 = 4;
pub const SOLVER_ITERATIONS: u32 = 8;
pub const MAX_SIM_SECONDS: f32 = 90.0;  // מעבר לזה → TIMEOUT
pub const WORLD_WIDTH: f32 = 1600.0;    // יחידות עולם (= 50m)
pub const WORLD_HEIGHT: f32 = 1200.0;   // (= 37.5m)
pub const GRID: f32 = 16.0;             // snap-to-grid
pub const SLEEP_ENABLED: bool = false;  // שובר דטרמיניזם בשרשראות ארוכות! חובה false
```

> **הערת קואורדינטות (v1.2):** `nannou` משתמש בקונבנציית +y-up סטנדרטית (כמו OpenGL/מתמטיקה רגילה) — למעלה על המסך הוא y חיובי. זה **הפוך** מהנחת היסוד של v1.1 (מבוססת Pixi.js, שהיא +y-down כמו רוב ה-canvas 2D של הדפדפן). המשמעות המעשית: כבידה היא `y = -9.81` (לא `+9.81`), ו"למעלה במסך" = y גדול. עדכן כל חישוב מצלמה/הצבה בהתאם. זה לא משנה שום החלטת gameplay — רק את הסימן המוסכם בקוד.

### 2.3 מכונת המצבים של המשחק

```
                                            ┌──────────────────────────────────────────┐
                                            │                                            │
                                       ┌────▼────┐    play     ┌─────────┐   win   ┌─────┴────┐
                                       │  EDIT   │────────────▶│ RUNNING │────────▶│  SOLVED  │
                                       └────▲────┘             └────┬────┘         └──────────┘
                                            │                       │ lose/timeout
                                            │ reset                 ▼
                                            │                  ┌─────────┐
                                            └──────────────────│ FAILED  │
                                            │                  └─────────┘
                                            │ resume       ┌─────────┐
                                            └──────────────│ PAUSED  │◀── pause ──┐
                                                            └─────────┘            │
                                                                       (from RUNNING)
```

| מצב | פיזיקה | גרף | קלט מותר |
|---|---|---|---|
| EDIT | קפואה (בלי `step()`) | לא מוערך | הצבה, הזזה, סיבוב, מחיקה, חיבור, pan/zoom |
| RUNNING | `step()` בקצב קבוע | מוערך כל tick | pause, reset, pan/zoom **בלבד** |
| PAUSED | קפואה | קפוא | resume, reset, pan/zoom |
| SOLVED | ממשיכה לרוץ 2s לצורך חגיגה | מוערך | next level, reset |
| FAILED | קפואה | קפוא | reset |

### 2.4 לולאת המשחק — מימוש מדויק (Rust, v1.2)

```rust
// core/sim/fixed_step_loop.rs
pub struct FixedStepLoop {
    accumulator: f32,
    sim_time: f32,
    tick_index: u64,
}

impl FixedStepLoop {
    /// קרוי פעם אחת לכל פריים רינדור. `real_dt_seconds` מגיע מ-nannou's `Update.since_last`.
    pub fn frame(&mut self, ctx: &mut SimContext, real_dt_seconds: f32) {
        if !matches!(ctx.state, GameState::Running | GameState::Solved) {
            return; // מציירים גם במצב קפוא — הקריאה לרנדר קורית בנפרד ב-view()
        }
        // clamp כדי למנוע death spiral אחרי מינימיזציה של האפליקציה
        self.accumulator += real_dt_seconds.min(0.1);

        let mut steps = 0;
        while self.accumulator >= FIXED_DT && steps < MAX_SUBSTEPS_PER_FRAME {
            self.tick(ctx);
            self.accumulator -= FIXED_DT;
            steps += 1;
        }
        // אם נשארנו מאחור — זורקים את השארית. עדיף לדלג מאשר לשבור דטרמיניזם.
        if steps == MAX_SUBSTEPS_PER_FRAME {
            self.accumulator = 0.0;
        }
        // self.accumulator / FIXED_DT זמין לרינדור כ-interpolation alpha
    }

    fn tick(&mut self, ctx: &mut SimContext) {
        // ⚠️ הסדר הזה קריטי לדטרמיניזם. אל תשנה אותו.
        ctx.energy_graph.propagate(&mut ctx.parts);      // 1. חשמל → אור/חום/סיבוב (טופולוגי)
        ctx.thermal_system.update(&mut ctx.parts, FIXED_DT); // 2. התקדמות פתילים, הצתות
        ctx.field_system.apply_forces(&mut ctx.physics);  // 3. רוח / ואקום / מגנט
        ctx.rope_network.solve_tension(&mut ctx.physics); // 4. אילוצי חבלים
        ctx.gear_train.apply_torques(&mut ctx.physics);   // 5. מומנטים למפרקים מנועיים
        ctx.physics.step();                               // 6. הצעד הפיזיקלי (rapier2d)
        ctx.collision_router.drain(&mut ctx.energy_graph); // 7. תרגום אירועי מגע ל-IMPACT
        ctx.win_conditions.evaluate(ctx);                 // 8. בדיקת ניצחון/כישלון

        self.sim_time += FIXED_DT;
        self.tick_index += 1;
        if self.sim_time > MAX_SIM_SECONDS {
            ctx.fail(FailReason::Timeout);
        }
    }
}
```

### 2.5 מנגנון Reset — הכלל הכי חשוב במסמך

לעולם אל תנסה "להחזיר" את הפיזיקה אחורה. Reset = בנייה מחדש של העולם מאפס.

```rust
pub fn reset(&mut self) {
    // rapier2d: RigidBodySet/ColliderSet הם ערכים רגילים (לא WASM handle) —
    // פשוט מחליפים אותם ב-instance חדש, ה-Drop הרגיל של Rust מטפל בניקוי.
    self.physics = Physics::new(GRAVITY);
    self.energy_graph.clear();
    self.rope_network.clear();
    // editor_state לעולם לא משתנה ב-Running. המקור היחיד לאמת.
    LevelLoader::build(&mut self.physics, &self.level, &self.editor_state);
    self.loop_state = FixedStepLoop::default();
    self.state = GameState::Edit;
}
```

לשם כך חייבים שני מבני נתונים נפרדים:

- **`editor_state`** — האמת. מה השחקן הציב, איפה, באיזו זווית, ומה מחובר למה. קריא בלבד ברגע שנכנסים ל-`Running`.
- **`physics` / `energy_graph` / וכו'** — נגזרים. גופי rapier2d, handles, מצבי גרף. נהרסים ונבנים מחדש בכל reset.

### 2.6 דטרמיניזם — צ'קליסט חובה לסוכן הקוד

1. `SLEEP_ENABLED = false` (יש להשבית שינה של גופים ברמת ה-`RigidBody` בעת היצירה).
2. סדר יצירה קבוע: מיין את `editor_state.parts` לפי `id` (`String`, `sort()`) לפני הבנייה. לעולם אל תסתמך על סדר איטרציה של `HashMap`/`HashSet` (לא מובטח ב-Rust!) — השתמש ב-`BTreeMap` או במיון מפורש.
3. אפס שימוש ב-`rand::random()` (או כל מקור רנדומליות אחר) בתוך `core::sim`/`core::graph`. אם צריך רנדומליות ויזואלית — מחוץ ל-tick, בשכבת הרינדור (`render/`) בלבד.
4. אפס תלות ב-`std::time::Instant::now()`/`SystemTime::now()` בתוך `tick()`. הזמן היחיד הוא `sim_time`.
5. אפס תלות ב-`real_dt_seconds` בתוך הלוגיקה — רק `FIXED_DT`.
6. איטרציות סולבר קבועות (`IntegrationParameters` לא משתנה בזמן ריצה).
7. בדיקה אוטומטית (`tests/determinism.rs`): אחרי 600 ticks, מצב כל הגופים (translation + rotation, מעוגל ל-6 ספרות) חייב להיות זהה בין 3 ריצות עצמאיות של אותו שלב. ראה גם snapshot roundtrip tests המובנים ב-`rapier2d` עצמו כהשראה.

### 2.7 תנאי ניצחון

```rust
// core/level/win_conditions.rs
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WinCondition {
    Contained { subject_tag: String, container_id: String, hold_ms: f32 },
    ReachedZone { subject_tag: String, zone_id: String },
    EnergyState { node_id: String, energy: EnergyType, active: bool, for_ms: f32 },
    Destroyed { target_id: String },
    AllOf { conditions: Vec<WinCondition> },
    AnyOf { conditions: Vec<WinCondition> },
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum FailCondition {
    Timeout,
    SubjectDestroyed { subject_tag: String },
    LeftBounds { subject_tag: String },
}
```

**כלל:** תנאי ניצחון בודק מצב עולם, לעולם לא הצבת רכיבים. זה מה שמאפשר פתרונות מרובים.

---

## 3. התאמה למובייל וממשק משתמש (Mobile UI/UX)

> **הערת סטטוס (עודכן ב-v1.3):** הסעיף הזה כולו נכתב סביב קלט מגע/נייד — ועם המעבר ל-Bevy (§2.1a), זו כבר לא "כוונה רחוקה למובייל" אלא **המכשיר שהמפתח עצמו ישחק עליו**, ומיועדת להיות רלוונטית הרבה יותר מוקדם ממה ש-v1.2 הניחה. סדר העבודה בפועל: M5 ("Desktop + touch input") ממש שתי שכבות קלט זו לצד זו מהתחלה — עכבר+מקלדת לפיתוח מהיר על מחשב (קליק=touch, גרירה=drag, גלגלת=zoom), ומחוות מגע אמיתיות (3.2–3.5 כלשונם) לבדיקה על מכשיר אנדרואיד אמיתי החל מ-M7 (§7.5) ולא מחכה ל-M12 כמו שתוכנן קודם. סעיף 3.9 (שפת תנועה, Juice, זהות חזותית) תקף במלואו ומיידית לשתי הפלטפורמות. Bevy מספק את שתי סוגי הקלט (`bevy::input::mouse`, `bevy::input::touch`) דרך אותו מנגנון אירועים — אין צורך בשני code paths נפרדים לגמרי, רק במיפוי מחוות שונה.

### 3.1 פריסת מסך

```
┌─────────────────────────────────┐ ← safe-area-top
│ ‹ המטרה ...שלב 12  🎯  ⓘ  🔄 │ HUD עליון — 56pt
├─────────────────────────────────┤
│                                   │
│                                   │
│         CANVAS (nannou)          │ שאר הגובה
│      pan / zoom / drop           │
│                                   │
│                                   │
├─────────────────────────────────┤
│ ⚙️ 🔥 💧 🔩 ▸▸▸ (גלילה אופקית)  │ ארגז החלקים — 96pt
├─────────────────────────────────┤
│         [  ▶ הפעל  ]             │ כפתור ראשי — 64pt
└─────────────────────────────────┘ ← safe-area-bottom
```

**החלטות מפתח:**

- הכפתור הראשי **תמיד באותו מקום** ומחליף תווית: הפעל → עצור → אפס. בלי כפתורים קופצים.
- ארגז החלקים הוא drawer שניתן לגרור למעלה למצב מורחב (רשת 4 עמודות) — כי 50 רכיבים לא נכנסים לגלילה אופקית אחת.
- **אזור בטוח לאגודל:** כל פעולה הרסנית (מחיקה, איפוס) לא בטווח 44pt מקצוות המסך התחתונים.
- מינימום touch target: **44×44pt**.

### 3.2 גרירה ושחרור (Drag & Drop)

```
STATE: IDLE → PICKING → DRAGGING → PLACING

IDLE:
  touchstart על פריט בארגז → טיימר 120ms → PICKING

PICKING (120–180ms):
  haptic: light impact
  הפריט "מתרומם" (scale 1.15, shadow, opacity 0.9)
  אם האצבע זזה >8px לפני ה-120ms: בטל ← זו גלילה של הארגז, לא הרמה.

DRAGGING:
  ⚠️ קריטי: הספרייט מוצג ב-(finger.y − 64px) כדי שהאגודל לא יסתיר אותו.
  קו מקווקו מחבר את האצבע לספרייט.
  הצללית (ghost) מוצגת במיקום ה-snap הסופי, לא במיקום האצבע.
  ירוק = הצבה חוקית | אדום = חפיפה/מחוץ לגבולות
  גרירה לקצה המסך (80px) → auto-pan של המצלמה

PLACING:
  touchend על מיקום חוקי → הצבה + haptic medium
  touchend על מיקום לא חוקי → אנימציית חזרה לארגז (250ms ease-out)
```

**מחיקת חיבור:** tap צף ✕ על החבל עצמו → מודגש → tap → מוחק. Swipe מהיר על החבל = גם מוחק.

### 3.3 Snap-to-Grid ומגנטיות

היררכיה של הצמדה, לפי סדר עדיפויות (הראשון שמתקיים מנצח):

| עדיפות | סוג | רדיוס | דוגמה |
|---|---|---|---|
| 1 | Anchor snap — נקודת עיגון תואמת | 28px | קצה חבל → יתד בגלגלת |
| 2 | Mesh snap — שיני גלגלים סמוך | 22px | `gear_small` → `gear_large` |
| 3 | Surface snap — הנחה על משטח | 18px | ארגז על קרש |
| 4 | Grid snap | תמיד | רשת 16px |

משוב מיידי: כשההצמדה פעילה, נקודת היעד מהבהבת + haptic selection feedback. חזותית חשוב יותר מדיוק.

### 3.4 בחירה, סיבוב והיפוך

`Tap` על רכיב מוצב → תפריט inline צף מעל הרכיב (לא מודאל, לא bottom sheet):

```
    ┌───────────────────────┐
    │ ↺  ↻  ⇄  ⇅  ⚙  🗑     │
    └───────────┬───────────┘
                ▼
            [ הרכיב ]
```

- ↺ ↻ — סיבוב בקפיצות לפי `rotationSnap` של הרכיב (90° לרוב, 15° למראות ולקרשים)
- ⇄ ⇅ — היפוך אופקי/אנכי (רק לרכיבים עם `flippable: true`)
- ⚙ — פתיחת inspector לפרמטרים (RPM של מנוע, עוצמת מאוורר, זווית שיגור)

**סיבוב חופשי:** שתי אצבעות על הרכיב הנבחר = rotation חופשי עם snap רך ל-15°. מד זווית מופיע במרכז.

### 3.5 כלי החיבור (Rope / Belt Tool)

1. השחקן בוחר `rope` מהארגז → נכנס ל-MODE_CONNECT
2. כל נקודות העיגון החוקיות בשלב מוארות בפולסים (הילה כחולה, רדיוס 24px)
3. נגיעה בעוגן א' → הנקודה ננעלת, קו גומי עוקב אחרי האצבע
4. עוגנים לא-חוקיים (אותו גוף, מרחק > maxLength) מעומעמים בזמן אמת
5. נגיעה בעוגן ב' → החבל נוצר, haptic success
6. נגיעה במקום ריק / כפתור ✕ → ביטול

### 3.6 מצלמה, Pan ו-Zoom

```ts
const CAMERA = {
  minZoom: 0.6,        // כל העולם נראה
  maxZoom: 3.0,
  defaultFit: 'CONTAIN', // בטעינת שלב — התאמה לגבולות + padding 24px
  panMomentum: 0.92,    // friction decay
  edgeResistance: 0.35, // "גומי" בגבולות העולם
};
```

| מחווה | ב-EDIT | ב-RUNNING |
|---|---|---|
| אצבע אחת על רקע | Pan | Pan |
| אצבע אחת על רכיב | גרירת הרכיב | Pan |
| שתי אצבעות | Pinch zoom + pan | Pinch zoom + pan |
| Double tap | zoom ל-1.5× סביב הנקודה | אותו דבר |
| Double tap ב-zoom > 1.5 | חזרה ל-fit | אותו דבר |

**Auto-follow ב-RUNNING:** אופציונלי בהגדרות. המצלמה עוקבת בעדינות (lerp 0.08) אחרי הגוף "הפעיל ביותר" — הגוף עם |velocity| הגבוה ביותר, עם היסטרזיס של 400ms כדי למנוע קפיצות. ברירת מחדל: כבוי — שחקנים מנוסים רוצים לראות הכול.

### 3.7 Responsive Viewport

עולם המשחק בגודל קבוע (1600×1200 יחידות). המצלמה מתאימה אותו למסך:

```ts
const scale = Math.min(screenW / WORLD_W, screenH_available / WORLD_H);
// טלפון צר (390×844): עדיין רואים את כל העולם ב-zoom ~0.55 → צריך zoom-in לעבודה
// טאבלט (1024×1366): רואים הכול בנוחות
```

- **טלפון (< 600pt):** ארגז תחתון, HUD מינימלי, עורך ברירת-מחדל ב-zoom 1.0 ממורכז על אזור המטרה.
- **טאבלט (≥ 600pt):** ארגז הופך לעמודה צדדית (ימין ב-RTL/שמאל ב-LTR) ברוחב 200pt, קנבס גדול יותר.
- **תמיכה בסיבוב מסך:** landscape הוא המצב המומלץ. אם ב-portrait — הצג רמז "סובב לחוויה טובה יותר" פעם אחת.
- **Safe areas:** `env(safe-area-inset-*)` לכל-הצדדים. בלי זה, ה-home indicator באייפון בולע את כפתור ההפעלה.

### 3.8 פידבק ונגישות

- **Haptics** (Capacitor Haptics): light בהרמה, selection בהצמדה, medium בהצבה, success בניצחון, warning בכישלון.
- **מצב האטה:** בזמן ריצה, slider ×0.25–×1 שמשנה כמה `tick()` קורים לפריים (לא את `SIM.FIXED_DT`!) — כך הדטרמיניזם נשמר והשחקן יכול לנתח מה השתבש. פיצ'ר קריטי לפאזלים מורכבים.
- **Colorblind-safe:** אין הסתמכות על צבע בלבד. חוקי/לא-חוקי גם בצורה (✓/✕), לא רק בירוק/אדום.
- **RTL:** אם יש לוקליזציה לעברית — ה-UI מתהפך, הקנבס לא. עולם המשחק תמיד LTR לוקליזציה תמיד עברית.

### 3.9 שפת תנועה, Juice ורמת גימור 2026 (חדש ב-1.1)

זהו הסעיף שמפריד "פרויקט פיזיקה נחמד" מ-"משחק שנראה כאילו יצא ב-2026". הכלל המנחה: **כל פעולה במשחק מקבלת תגובה חזותית תוך פחות מ-80ms**, בלי יוצא מן הכלל.

#### 3.9.1 Design Tokens — מקור אמת יחיד לעיצוב

בדיוק כמו ש-`SIM` הוא מקור אמת אחד לפיזיקה, `UI_TOKENS` הוא מקור אמת אחד לעיצוב — שום קומפוננטה לא "ממציא" ערך משלה:

```ts
// ui/tokens.ts
export const MOTION = {
  instant:  { duration: 90,  easing: 'cubic-bezier(0.4, 0, 0.2, 1)' },  // toggle, בחירה
  snappy:   { duration: 160, easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)' }, // spring overshoot — הצבת רכיב
  smooth:   { duration: 240, easing: 'cubic-bezier(0.22, 1, 0.36, 1)' },   // מעברי מסך
  celebrate:{ duration: 600, easing: 'cubic-bezier(0.16, 1, 0.3, 1)' },    // ניצחון
};

export const ELEVATION = { // בלוברי-אור, לא צל שחור שטוח
  resting:  { blur: 8,  alpha: 0.18, y: 2 },
  dragging: { blur: 24, alpha: 0.32, y: 10, glowBoost: 1.4 },
  modal:    { blur: 40, alpha: 0.45, y: 16 },
};

export const RADIUS = { sm: 8, md: 14, lg: 22, pill: 999 };
export const SPACE  = [0, 4, 8, 12, 16, 24, 32, 48]; // scale, לא מספרים חופשיים
```

**כלל לסוכן הקוד:** אף `duration`, `easing`, `border-radius` או `box-shadow` לא נכתב inline בקומפוננטה. הכול דרך `MOTION` / `ELEVATION` / `RADIUS`. זה מה שנותן תחושת "מערכת אחת קוהרנטית" ולא אוסף של אנימציות שהומצאו בנפרד.

#### 3.9.2 קידוד צבע לפי סוג אנרגיה — הליבה של השפה החזותית

**הרעיון המרכזי:** כל `EnergyType` (סעיף 1.2) מקבל צבע חתימה קבוע, המשמש **בו-זמנית** למטרה פונקציונלית (השחקן קורא את גרף האנרגיה בעין) ולמטרה אסתטית (זה מה שנותן את מראה ה"מעבדה קינטית" הזוהרת):

| EnergyType | צבע חתימה | HEX | שימוש חזותי |
|---|---|---|---|
| ROTARY | ענבר חם | `#F2A33D` | קו מקווקו מסתובב סביב צירים פעילים |
| TENSION | ציאן חשמלי | `#3DDBD9` | ליבת האור בתוך חבלים מתוחים |
| ELECTRIC | צהוב-לימון זוהר | `#F4E04D` | ניצוצות זעירים על מוליכים פעילים |
| THERMAL | כתום-אדום | `#F2542D` | גרדיאנט זוהר סביב מקור חום, טרמוגרפי |
| PNEUMATIC | טורקיז בהיר שקוף | `#7FE0C9` | קווי זרימה מונפשים (particles) בכיוון הרוח |
| LIGHT | לבן-זהוב | `#FFF3D6` | קרן volumetric רכה עם bloom עדין |
| IMPACT | מגנטה חשמלי | `#E23D9C` | פרץ טבעת (ring burst) חד-פעמי בנקודת המגע |

בפועל: **כל קשת בגרף האנרגיה (חבל, ציר מסתובב, חוט חשמל) מצוירת עם shader של "זרימת חלקיקים"** — נקודות אור קטנות שנעות במהירות יחסית לעוצמת האות, בצבע החתימה של סוג האנרגיה שלהן. זה ממחיש חזותית בדיוק את מה ש-`EnergyGraph.propagate()` עושה בקוד — הרנדר "מספר את הסיפור" של הסימולציה במקום להסתיר אותו.

#### 3.9.3 כיוון רינדור: "וקטור עמוק" (Deep Flat)

לא פלטוני-שטוח (כמו אייקונים בסיסיים), ולא פסאודו-תלת-ממד ריאליסטי. אזור ביניים שהוא חתימת 2026:

- **צורות:** גיאומטריה וקטורית נקייה עם פינות מעוגלות עדינות (radius יחסי לגודל הרכיב, לא קבוע).
- **תאורה:** shader אור אחד גלובלי לכל הסצנה (מקור אור עליון-שמאלי, זווית קבועה) שיוצר highlight רך עליון + AO (ambient occlusion) עדין בבסיס כל רכיב — נותן נפח בלי טקסטורות. ב-nannou/wgpu: shader מותאם (WGSL, normal-map דמוי, לא תאורה תלת-ממדית אמיתית).
- **חומריות ללא זהות:** מתכת = ספקולר צר וחד; עץ/פלסטיק = ספקולר רחב ועמום; זכוכית = שקיפות + רפרקציה קלה. זו התחושה של TIM (חומרים שונים מתנהגים אחרת ויזואלית) בלי לשחזר עיצוב ספציפי שלו.
- **רקע:** שכבות parallax איטיות (2–3 שכבות, מהירויות שונות בזמן pan) — מרחב מופשט (לא "סדנה", לא "מעבדה מזוהה") בגרדיאנט עדין, לא צבע אחיד שטוח.
- **פוסט-פרוססינג עדין:** vignette רך, bloom נמוך-עוצמה על אלמנטים זוהרים בלבד (לא גלובלי — פוגע בקריאות במסך קטן), chromatic aberration כמעט בלתי מורגש בפיצוצים בלבד.

#### 3.9.4 פלטת בסיס (ניטרלית, לא "סדנה")

```
--bg-canvas:     #14171F   /* קנבס כהה, לא שחור מוחלט */
--bg-panel:      #1D212C   /* פאנלים */
--surface-glass: rgba(30, 34, 46, 0.68)  /* HUD/ארגז — זכוכית מטושטשת */
--ink-primary:   #F4F6FB
--ink-secondary: #9AA3B8
--accent-brand:  #6C8CFF   /* מיתוג/CTA כללי — לא קשור לאנרגיה */
--success:       #35D399
--danger:        #FF5C6C
```

**מצב בהיר (אופציונלי, נגישות):** אותה מערכת בהיפוך טונלי — `--bg-canvas: #EFF1F6`, שומר את אותם צבעי אנרגיה (הם עובדים על שתי הרקעים כי הם רוויים ובהירים דיו).

#### 3.9.5 HUD/UI כשכבת זכוכית צפה (Glass HUD)

הפאנלים (HUD עליון, ארגז חלקים, inspector) הם משטחי `surface-glass` עם blur (`backdrop-filter: blur(18px)`), לא לוחות אטומים — כדי שהקנבס (המשחק עצמו) יישאר "הכוכב" גם כשה-UI פתוח. כפתורים ראשיים הם pill-shaped עם gradient עדין ו-inner-glow בעת הפעלה.

#### 3.9.6 Juice — רשימת תגובות חובה

| אירוע | תגובה חזותית | תגובה קולית/הפטית |
|---|---|---|
| הצבת רכיב | `snappy` scale-bounce 1.0→1.08→1.0 + טבעת אור קצרה בצבע קטגוריית הרכיב | haptic medium, "clack" רך |
| חיבור חבל/רצועה נוצר | קו מצייר את עצמו מקצה לקצה (200ms) + פעימת אור אחת | haptic success |
| לחיצה על "הפעל" | HUD דוהה החוצה חלקית, הקנבס מקבל vignette עדין ("focus mode") | — |
| אנרגיה זורמת ראשונה בצינור/חבל | חלקיקי הצבע הרלוונטי "מתעוררים" בהדרגה (fade-in 300ms) — לא מופיעים בבת אחת | — |
| פגיעה/שבירה | squash-stretch לפי מסה + shard particles קטנים בצבע החומר | haptic warning קל |
| ניצחון | `celebrate` — כל הרכיבים שבפתרון מהבהבים פעימת אור אחת מסונכרנת, קונפטי וקטורי דליל (לא גרפי מלא מסך), HUD חוזר בהדרגה | haptic success כפול |
| כישלון/טיימאאוט | fade-to-desaturate של הסצנה (400ms), לא פופ-אפ אדום פתאומי | haptic warning |

**עיקרון-על ל-Juice:** תגובה חזותית קודמת לטקסט. לפני שמופיע "פתרת!" בכל מקרה כבר ראית את האור מהבהב. UI טקסטואלי הוא אישור למה שכבר הרגשת, לא המקור למידע.

#### 3.9.7 טיפוגרפיה ואייקונוגרפיה

- **כותרות/HUD:** Sans גיאומטרי עם משקל בינוני-כבד (למשל משפחת פונט בסגנון Inter/General Sans — לבחור פונט חינמי/Google Fonts קונקרטי בזמן המימוש, לא ממותג).
- **מספרים (RPM, זמן, מונים):** Mono עם tabular figures — כדי שמספרים מתעדכנים לא "יקפצו" ברוחב.
- **אייקונים:** קו אחיד (stroke-based, לא filled), עובי קו יחסי לגודל — לא סט מעורב של סגנונות.

#### 3.9.8 השוואת כיווני אמנות — למה Prism Foundry נבחר

| כיוון | תחושת "2026 פרימיום" | קריאות במסך קטן | ניטרליות משפטית | הכרעה |
|---|---|---|---|---|
| **Prism Foundry** (מעבדה קינטית, וקטור עמוק + קידוד צבע לפי אנרגיה) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ✅ **נבחר** |
| Timber & Brass (סדנת נגרות — כיוון v1.0) | ⭐⭐⭐ (נעים אך נוסטלגי) | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | חלופה — ראה 4.3 |
| Neon Lab | ⭐⭐⭐⭐ | ⭐⭐ (רכיבים דומים מתערבבים) | ⭐⭐⭐⭐⭐ | חלופה |
| Paper Machine | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | חלופה |

**הנימוק:** Prism Foundry הוא היחיד שהופך מגבלה טכנית (צריך להבחין בין 7 סוגי אנרגיה בגרף לוגי) להזדמנות אמנותית (קידוד צבע = גם פונקציונלי וגם "יפה בעצמו"), ומייצר בדיוק את מראה ה"מוצר טכנולוגי מלוטש" שמצפים לו ממשחק שיוצא ב-2026 — לא סגנון תקופתי (סדנת נגרות), לא סגנון נישתי (נייר מקופל), לא סגנון עם בעיית קריאות (ניאון).

---

## 4. מנגנון מניעת הפרת זכויות יוצרים (Legal & Branding Safety)

### 4.1 מה מוגן ומה לא — הקו המנחה

| ✅ מותר להשתמש (לא מוגן) | ❌ אסור בהחלט |
|---|---|
| מכניקות ורעיונות משחק (חוקי משחק אינם מוגנים בזכויות יוצרים) — **ובכלל זה: כמה שיותר קרוב ביחסים הפיזיקליים והתפקודיים, ראה עיקרון-העל בסעיף 1.1** | השם "The Incredible Machine" / "TIM" / "Contraptions" |
| הקונספט של פאזל תגובת-שרשרת | כל ספרייט, טקסטורה, צליל או מוזיקה מהמקור |
| חוקי פיזיקה (גלגלת, גלגל שיניים, פתיל) | שמות הדמויות: Pokey, Mort, Mel, Ernie, Professor Tim, Curie, Newton, Bik, Sid |
| טיפוסי רכיבים גנריים (כדור, מאוורר, נר) | פריסות שלבים מקוריות — אלה יצירה מוגנת |
| סוגת "פאזל פיזיקלי" | קוד מקור, קבצי נתונים, assets של reverse-engineering |

### 4.2 שלוש מלכודות שחייבים להכיר

1. **"Rube Goldberg" הוא סימן מסחר רשום** (Rube Goldberg Inc.). אל תשתמש בו בשם האפליקציה, בכותרת בחנות, ב-subtitle או ב-keywords. אפשר לתאר את המשחק כ-"chain-reaction contraption puzzles" — זה תיאור גנרי ובטוח.
2. **מטא-דאטה בחנות = הסיכון הגדול ביותר.** אל תכתוב "כמו The Incredible Machine", "clone של TIM", "מחווה ל-" בתיאור או ב-ASO keywords. שימוש בסימן מסחר של אחר במטא-דאטה הוא בדיוק מה שגורם להורדות מהחנות. גם בעמוד ה-Press Kit שלך — לא.
3. **אין שמות של מותגים בתוך המשחק.** אין "Coca-Cola can", אין דמות שנראית כמו דמות מוכרת. כל נכס חזותי — מקורי או ברישיון מפורש (רכישה מסחרית עם חשבונית / CC0), מכורת, מקורי או ברישיון מפורש.

### 4.3 תמה ויזואלית — ההמלצה

> **עדכון 1.1:** ההמלצה הראשית עברה מ-"Timber & Brass" ל-**"Prism Foundry"** (ראה פירוט מלא ומנומק בסעיף 3.9). שלוש האפשרויות המקוריות נשמרות כאן כתיעוד ההיסטוריה של ההחלטה ותקפות כחלופות.

#### ✅ ההמלצה הנוכחית: "Prism Foundry" — מעבדה קינטית פרימיום (סעיף 3.9)

וקטור שטוח-עמוק עם תאורה רכה מכיוון גלובלי אחד, כל סוג אנרגיה מקבל צבע חתימה עקבי שמונפש כזרימת חלקיקים לאורך חבלים/צירים/מוליכים, רקע abstract כהה עם parallax, HUD כזכוכית מטושטשת צפה. אפס זיקה לסגנון תקופתי או תעשייתי מסוים — נראה כמו מוצר טכנולוגי, לא כמו מוזיאון.

**פלטת בסיס:** `#14171F` (קנבס) / `#1D212C` (פאנלים) / `#6C8CFF` (מיתוג) — ראה טבלת צבעי אנרגיה המלאה בסעיף 3.9.4.

**טיפוגרפיה:** Sans גיאומטרי לכותרות + Mono לערכים מספריים (tabular figures).

**למה זו הבחירה:** מקסימלית מבחינת "תחושת 2026" (ראה השוואה בסעיף 3.9.8), עדיין ניטרלית משפטית לחלוטין (לא נשענת על שום סגנון קיים — לא TIM, לא שום דבר אחר), והיא היחידה שהופכת את שכבת גרף האנרגיה (הליבה הטכנית של המשחק) לבסיס השפה החזותית עצמה.

#### חלופה א' (הומלצה ב-v1.0): "Timber & Brass" — סדנת אומן מודרנית

עולם של שולחן עבודה גדול בסדנת נגרות/מסגרות: עץ אלון, פליז מבריק, ברגים, מלחציים. הפאזל "יושב" על קיר לוח-כלים (pegboard) — מה שנותן הצדקה דיאגטית מושלמת לרשת ההצמדה ולנקודות העיגון. סגנון: וקטור שטוח עם צללים רכים וספקולר עדין על מתכת, לא cartoon.

פלטת צבעים: רקע #2A2420 (עץ כהה) / משטח #0A5C9D (עץ בהיר) / פליז #C9973F / מבטא #FA7A13 (טורקיז) / סכנה #D9584B.

**חיסרון מול Prism Foundry:** תחושה נעימה ונוסטלגית-מלאכתית, אבל פחות "מוצר 2026" ויותר "מוצר תקופתי/עונתי".

#### חלופה ב': "Neon Lab" — מעבדה עתידנית

רקע כהה, רכיבים זוהרים, אפקטי אור חזקים. יתרון: אור/לייזר נראים מדהים. **חיסרון:** קשה להבחין בין רכיבים דומים במסך קטן — בעיה קריטית לפאזל שבו קריאות היא הכול.

#### חלופה ג': "Paper Machine" — עולם נייר מקופל

כל הרכיבים כאוריגמי/קרטון עם צללים. יתרון: ייחודי מאוד, קל לייצור assets. חיסרון: פחות נעים, פחות קריא למכניקות מתכתיות.

### שמות מוצע לאפליקציה

Chainworks · Cogwright · Brasswork · Kludge · Tinker Hollow · Clank & Cogs
בעברית: גלגל שן · שרשרת · קליק-קלאק

> לפני נעילת שם: בדוק ב-USPTO TESS, EUIPO, מרשם סימני המסחר הישראלי, ובחיפוש ישיר ב-App Store ו-Google Play. וקנה את הדומיין.

### 4.4 טבלת שמות חלופיים — מקור → חדש

| קטגוריה | המקור (אין להשתמש) | השם החדש | הצדקה |
|---|---|---|---|
| דמות ראשית | Professor Tim | "The Foreman" / דמות ידיים בלבד | אין דמות מזוהה, רק כפפות עבודה שמצביעות במדריך |
| בעל חיים נע | Mort the Mouse | `walker_unit` — "ווקר" | רובוט צועד קטן. אותה מכניקה, אפס דמיון חזותי |
| טורף/מכשול | Pokey the Cat / Ernie the Alligator | `sentry_block` — "סנטרי" | בוט חוסם סטטי שדוחף |
| פיתיון | Cheese | `walker_beacon` — "משואה" | משואת אור שמושכת את הווקר |
| מקור כוח חי | Mouse Motor / Monkey Bike | `flywheel_spring` — "גלגל תנופה" | מנגנון קפיצי דרוך. מסיר לגמרי את זווית בעלי החיים |
| נשק | Revolver / Super Phazer | `punch_arm` + `launcher_tube` | הסרת נשק חם — גם משפטית וגם לדירוג גיל נמוך יותר |
| חגים | Christmas Tree, Pumpkin, Valentine Balloon | לא נכללים | תמות חג = עומס רישוי ותרבות מיותר |
| יעד | Bob's Fish Bowl / Laundry Basket | `bin_target` — "דלי איסוף" | גנרי לחלוטין |

**כלל לסוכן הקוד:** בכל הקוד, ה-assets וקבצי השלבים — אסור שיופיע אף אחד מהשמות בעמודה "המקור". גם לא בהערות, גם לא בשמות קבצים, גם לא ב-git commit messages.

### 4.5 קווים מנחים לעיצוב שלבים מקוריים

אל תעתיק מפות. תעתיק את עקומת הלמידה. ניתוח הפדגוגיה של הז'אנר נותן את המבנה הבא:

**מבנה הקמפיין — 60 שלבים**

| חטיבה | שלבים | תפקיד | מגבלת רכיבים |
|---|---|---|---|
| A — יסודות | 1–12 | רכיב חדש אחד לכל שלב, פתרון יחיד כמעט מובן מאליו | 1–3 |
| B — צירופים | 13–28 | שילוב 2–3 מכניקות שנלמדו. מופיע הפתרון המרובה הראשון | 3–6 |
| C — טיימינג | 29–44 | פתילים, השהיות, סדר אירועים. הפתרון נכון רק בעיתוי הנכון | 5–9 |
| D — מאסטר | 45–60 | שרשראות ארוכות, רכיבים "מיותרים" מטעים, אילוצי מרחב | 8–14 |

**כללי הזהב לתכנון שלב**

1. **חוק הרכיב החדש:** רכיב מוצג לראשונה בשלב שבו הוא הפתרון היחיד האפשרי. רק בשלב הבא הוא נהיה חלק מצירוף.
2. **חוק 3 הצעדים:** שלב טוב דורש 3–7 קשרים סיבתיים. פחות = טריוויאלי. יותר = מתסכל בנייד.
3. **חוק "הרגע":** לכל שלב צריכה להיות "תובנה" אחת ברורה — הרגע שבו השחקן מבין. אם אין רגע כזה — השלב הוא עבודת כפיים, לא פאזל.
4. **חוק העודף:** בשלבים C ו-D, תן רכיב אחד או שניים שאינם נחוצים. זה מייצר את החיפוש האמיתי.
5. **חוק הפתירות:** כל שלב חייב לפחות 2 פתרונות מאומתים שנשמרים כ-golden replay tests. אם יש רק אחד — התנאי צר מדי, הרחב אותו.
6. **חוק המסך הקטן:** כל השלב חייב להיות קריא ב-zoom 1.0 על מסך 390pt. אם צריך לגלול כדי להבין את המטרה — עצב מחדש.

**תהליך יצירת שלב (לסוכן הקוד)**

1. בחר מכניקה-מטרה (למשל: "חבל דרך גלגלת מרים משקולת")
2. הגדר את מצב הסיום (win condition) — לא את הדרך
3. בנה את הגיאומטריה הקבועה (fixed parts) שחוסמת את הפתרון הטריוויאלי
4. קבע את ארגז החלקים — המינימום הנדרש + 0–2 מטעים
5. פתור בעצמך פעמיים בדרכים שונות → שמור כ-`solutions[]`
6. הרץ את שני הפתרונות ב-headless → שניהם חייבים לעבור
7. הרץ 20 הצבות אקראיות → אף אחת לא אמורה לעבור

---

## 5. מבנה הנתונים והקוד (Data Structure for Claude Code)

### 5.1 JSON Schema — הגדרת שלב

שמור כ-`schemas/level.schema.json`. כל קובץ שלב עובר ולידציה מולו בטעינה + ב-CI.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://chainworks.game/schemas/level.schema.json",
  "title": "ChainWorks Level",
  "type": "object",
  "required": ["schemaVersion", "id", "title", "world", "fixedParts", "partsBin", "winConditions"],
  "additionalProperties": false,
  "properties": {
    "schemaVersion": { "const": 1 },
    "id": { "type": "string", "pattern": "^lvl_[a-z0-9_]+$" },
    "title": { "type": "string", "maxLength": 48 },
    "chapter": { "type": "string", "enum": ["A_FOUNDATIONS", "B_COMBOS", "C_TIMING", "D_MASTER"] },
    "order": { "type": "integer", "minimum": 1 },
    "difficulty": { "type": "integer", "minimum": 1, "maximum": 5 },
    "goalText": { "type": "string", "maxLength": 120,
      "description": "מוצג ב-HUD. חייב לתאר תוצאה, לא שיטה." },
    "hints": { "type": "array", "items": { "type": "string" }, "maxItems": 3 },
    "world": {
      "type": "object",
      "required": ["width", "height"],
      "additionalProperties": false,
      "properties": {
        "width": { "type": "number", "default": 1600 },
        "height": { "type": "number", "default": 1200 },
        "gravityY": { "type": "number", "default": 9.81 },
        "timeLimitSec": { "type": "number", "default": 90 },
        "theme": { "type": "string", "default": "prism_foundry" }
      }
    },
    "fixedParts": {
      "description": "רכיבים שהשחקן לא יכול להזיז או למחוק",
      "type": "array",
      "items": { "$ref": "#/definitions/placedPart" }
    },
    "preplacedParts": {
      "description": "רכיבים מוצבים מראש שהשחקן *כן* יכול להזיז",
      "type": "array",
      "items": { "$ref": "#/definitions/placedPart" }
    },
    "partsBin": {
      "description": "המלאי הזמין לשחקן",
      "type": "array",
      "items": {
        "type": "object",
        "required": ["partType", "count"],
        "additionalProperties": false,
        "properties": {
          "partType": { "type": "string" },
          "count": { "type": "integer", "minimum": 1, "maximum": 99 },
          "lockedParams": {
            "type": "array", "items": { "type": "string" },
            "description": "פרמטרים שהשחקן לא יכול לשנות בשלב הזה"
          }
        }
      }
    },
    "connections": {
      "description": "חיבורים קבועים מראש (חבלים/רצועות)",
      "type": "array",
      "items": { "$ref": "#/definitions/connection" }
    },
    "winConditions": {
      "type": "array", "minItems": 1,
      "items": { "$ref": "#/definitions/condition" }
    },
    "failConditions": {
      "type": "array",
      "items": { "$ref": "#/definitions/condition" }
    },
    "solutions": {
      "description": "פתרונות מאומתים — משמשים ל-golden replay tests ולרמזים",
      "type": "array", "minItems": 1,
      "items": {
        "type": "object",
        "required": ["label", "parts"],
        "properties": {
          "label": { "type": "string" },
          "parts": { "type": "array", "items": { "$ref": "#/definitions/placedPart" } },
          "connections": { "type": "array", "items": { "$ref": "#/definitions/connection" } },
          "expectedSolveTick": { "type": "integer",
            "description": "tick שבו מצופה ניצחון (סובלנות ±10%)" }
        }
      }
    }
  },
  "definitions": {
    "placedPart": {
      "type": "object",
      "required": ["id", "partType", "x", "y"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z0-9_]+$" },
        "partType": { "type": "string" },
        "x": { "type": "number" },
        "y": { "type": "number" },
        "rotation": { "type": "number", "default": 0, "description": "מעלות, 0–359" },
        "flipX": { "type": "boolean", "default": false },
        "flipY": { "type": "boolean", "default": false },
        "scale": { "type": "number", "default": 1, "minimum": 0.5, "maximum": 2 },
        "tags": { "type": "array", "items": { "type": "string" },
          "description": "תגיות לוגיות לתנאי ניצחון, למשל SUBJECT" },
        "params": {
          "type": "object",
          "description": "פרמטרים ספציפיים לרכיב",
          "properties": {
            "rpm": { "type": "number", "minimum": 1, "maximum": 600 },
            "direction": { "type": "string", "enum": ["CW", "CCW"] },
            "power": { "type": "number", "minimum": 1, "maximum": 5 },
            "angle": { "type": "number", "minimum": 0, "maximum": 359 },
            "startsOn": { "type": "boolean" },
            "startsLit": { "type": "boolean" },
            "length": { "type": "number", "minimum": 10, "maximum": 600 },
            "charge": { "type": "integer", "minimum": 1, "maximum": 5 },
            "mode": { "type": "string", "enum": ["toggle", "momentary"] }
          },
          "additionalProperties": false
        }
      }
    },
    "connection": {
      "type": "object",
      "required": ["id", "kind", "from", "to"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string" },
        "kind": { "type": "string", "enum": ["ROPE", "BELT", "WIRE"] },
        "from": { "$ref": "#/definitions/anchorRef" },
        "to": { "$ref": "#/definitions/anchorRef" },
        "maxLength": { "type": "number",
          "description": "BELT/ROPE בלבד. אם חסר — מחושב מהמרחק ההתחלתי ×1.05" },
        "routedThrough": {
          "type": "array", "items": { "$ref": "#/definitions/anchorRef" },
          "description": "גלגלות שהחבל עובר דרכן, לפי סדר"
        }
      }
    },
    "anchorRef": {
      "type": "object",
      "required": ["partId"],
      "additionalProperties": false,
      "properties": {
        "partId": { "type": "string" },
        "anchorIdx": { "type": "integer", "minimum": 0, "default": 0 }
      }
    },
    "condition": {
      "type": "object",
      "required": ["type"],
      "properties": {
        "type": { "type": "string",
          "enum": ["CONTAINED", "REACHED_ZONE", "ENERGY_STATE", "DESTROYED",
                   "TIMEOUT", "SUBJECT_DESTROYED", "LEFT_BOUNDS", "ALL_OF", "ANY_OF"] },
        "subjectTag": { "type": "string" },
        "subjectId": { "type": "string" },
        "containerId": { "type": "string" },
        "zoneId": { "type": "string" },
        "targetId": { "type": "string" },
        "nodeId": { "type": "string" },
        "energy": { "type": "string",
          "enum": ["ROTARY", "TENSION", "ELECTRIC", "THERMAL", "PNEUMATIC", "LIGHT", "IMPACT"] },
        "active": { "type": "boolean" },
        "holdMs": { "type": "number", "default": 500 },
        "conditions": { "type": "array", "items": { "$ref": "#/definitions/condition" } }
      }
    }
  }
}
```

### 5.2 דוגמת שלב מלאה

`levels/A/lvl_a03_pulley_lift.json`

```json
{
  "schemaVersion": 1,
  "id": "lvl_a03_pulley_lift",
  "title": "משיכה מלמעלה",
  "chapter": "A_FOUNDATIONS",
  "order": 3,
  "difficulty": 1,
  "goalText": "הכנס את כדור העץ לדלי.",
  "hints": [
    "כדור כבד שנופל יכול למשוך משהו אחר למעלה.",
    "גלגלת משנה את כיוון המשיכה."
  ],
  "world": { "width": 1600, "height": 1200, "gravityY": 9.81, "timeLimitSec": 45 },
  "fixedParts": [
    { "id": "ground", "partType": "floor_ground", "x": 800, "y": 1180 },
    { "id": "ledge_l", "partType": "plank_wood", "x": 300, "y": 400, "rotation": 0 },
    { "id": "wall_mid", "partType": "wall_brick", "x": 800, "y": 800, "rotation": 90 },
    { "id": "bucket", "partType": "bin_target", "x": 1250, "y": 1100 },
    { "id": "shelf_r", "partType": "plank_wood", "x": 1250, "y": 300, "rotation": 0 }
  ],
  "preplacedParts": [
    { "id": "subject", "partType": "ball_wood", "x": 300, "y": 360, "tags": ["SUBJECT"] }
  ],
  "partsBin": [
    { "partType": "pulley_wheel", "count": 1 },
    { "partType": "rope", "count": 1 },
    { "partType": "ball_lead", "count": 1 },
    { "partType": "lever_seesaw", "count": 1 },
    { "partType": "plank_wood", "count": 2 }
  ],
  "winConditions": [
    { "type": "CONTAINED", "subjectTag": "SUBJECT", "containerId": "bucket", "holdMs": 600 }
  ],
  "failConditions": [
    { "type": "LEFT_BOUNDS", "subjectTag": "SUBJECT" },
    { "type": "TIMEOUT" }
  ],
  "solutions": [
    {
      "label": "גלגלת קלאסית",
      "parts": [
        { "id": "s_pul", "partType": "pulley_wheel", "x": 300, "y": 200 },
        { "id": "s_lead", "partType": "ball_lead", "x": 560, "y": 180 },
        { "id": "s_pl1", "partType": "plank_wood", "x": 900, "y": 560, "rotation": 20 }
      ],
      "connections": [
        { "id": "c1", "kind": "ROPE", "from": { "partId": "s_lead" },
          "to": { "partId": "subject" }, "routedThrough": [{ "partId": "s_pul" }],
          "maxLength": 420 }
      ],
      "expectedSolveTick": 480
    },
    {
      "label": "מנוף ומדרון",
      "parts": [
        { "id": "s_lev", "partType": "lever_seesaw", "x": 450, "y": 900, "rotation": 0 },
        { "id": "s_ld2", "partType": "ball_lead", "x": 380, "y": 200 },
        { "id": "s_pl2", "partType": "plank_wood", "x": 950, "y": 700, "rotation": 25 },
        { "id": "s_pl3", "partType": "plank_wood", "x": 1150, "y": 900, "rotation": 15 }
      ],
      "connections": [],
      "expectedSolveTick": 620
    }
  ]
}
```

### 5.3 קטלוג הרכיבים כ-data

כל רכיב הוא קובץ — `data/parts/ball_rubber.json`. `PartRegistry` טוען את כולם.

```json
{
  "partType": "ball_rubber",
  "displayKey": "part.ball_rubber",
  "category": "DYNAMIC",
  "tier": "P0",
  "body": {
    "type": "dynamic",
    "shape": { "kind": "ball", "radius": 14 },
    "mass": 0.6,
    "restitution": 0.85,
    "friction": 0.80,
    "linearDamping": 0.01,
    "angularDamping": 0.02,
    "gravityScale": 1.0,
    "ccd": true
  },
  "tags": ["BOUNCY"],
  "windFactor": 1.0,
  "ports": [],
  "anchors": [],
  "editor": {
    "rotatable": false,
    "rotationSnap": 0,
    "flippable": false,
    "sprite": "parts/ball_rubber.png",
    "binIcon": "icons/ball_rubber.svg",
    "hitboxPadding": 10
  },
  "params": {}
}
```

```json
{
  "partType": "motor_electric",
  "displayKey": "part.motor_electric",
  "category": "POWER",
  "tier": "P0",
  "body": {
    "type": "fixed",
    "shape": { "kind": "box", "w": 48, "h": 48 }
  },
  "tags": ["METALLIC"],
  "ports": [
    { "id": "pwr", "dir": "IN", "energy": "ELECTRIC", "offset": { "x": -24, "y": 0 } },
    { "id": "out", "dir": "OUT", "energy": "ROTARY", "offset": { "x": 24, "y": 0 } }
  ],
  "anchors": [
    { "idx": 0, "kind": "ROTARY", "offset": { "x": 24, "y": 0 } }
  ],
  "editor": {
    "rotatable": true, "rotationSnap": 90, "flippable": true,
    "sprite": "parts/motor.png", "binIcon": "icons/motor.svg"
  },
  "params": {
    "rpm": { "type": "number", "min": 30, "max": 600, "step": 30, "default": 180 },
    "direction": { "type": "enum", "values": ["CW", "CCW"], "default": "CW" }
  }
}
```

### 5.4 פורמט שמירת התקדמות

```json
{
  "schemaVersion": 1,
  "profileId": "local",
  "levels": {
    "lvl_a03_pulley_lift": {
      "solved": true,
      "attempts": 7,
      "bestPartCount": 3,
      "bestSolveTick": 455,
      "savedBuild": { "parts": [], "connections": [] },
      "solvedAt": "2026-09-12T10:22:41Z"
    }
  },
  "sandboxes": [
    { "id": "sbx_01", "name": "הניסוי שלי", "updatedAt": "...",
      "data": { "world": {}, "parts": [], "connections": [] } }
  ],
  "settings": {
    "haptics": true, "sound": true, "music": true,
    "cameraAutoFollow": false, "gridVisible": true, "locale": "he"
  }
}
```

### 5.5 ארכיטקטורת הקוד (מעודכן v1.2 — Rust, בהשראת OpenTIM)

#### 5.5.1 המלצת פלטפורמה — הנימוק המלא (מעודכן v1.2)

| קריטריון | Rust + nannou + rapier2d | TS + Pixi + Rapier-WASM (v1.1) | Unity |
|---|---|---|---|
| התאמה ל-Claude Code | ⭐⭐⭐⭐⭐ קובצי `.rs` פשוטים, בלי build-tool config מורכב | ⭐⭐⭐⭐⭐ | ⭐⭐ פרויקט מבוסס editor ו-`.meta` files |
| דטרמיניזם | ⭐⭐⭐⭐⭐ מובטח, native, אין שכבת WASM | ⭐⭐⭐⭐⭐ מובטח (אבל דרך WASM) | ⭐⭐ |
| מהירות איטרציה | ⭐⭐⭐⭐⭐ `cargo run` מקומי, אין build-step דפדפן/WASM | ⭐⭐⭐⭐⭐ רענון דפדפן, גם מהנייד | ⭐⭐ צריך build |
| ביצועים ~200 גופים | ⭐⭐⭐⭐⭐ native, בלי overhead WASM | ⭐⭐⭐⭐ מספיק (WASM) | ⭐⭐⭐⭐⭐ בהחלט |
| נאמנות ארכיטקטונית ל-OpenTIM | ⭐⭐⭐⭐⭐ אותה שפה, מודולים באותה צורה (`part.rs` וכו') | ⭐⭐ שפה שונה לגמרי | ⭐ |
| מובייל/App Store מהיום הראשון | ⭐ `nannou` לא תומך במובייל — שלב 2 נפרד (§7.5) | ⭐⭐⭐⭐⭐ Capacitor מיידי | ⭐⭐⭐⭐⭐ |

**הכרעה: Rust.** לאחר סקירת [OpenTIM](https://github.com/mrfixit2001/OpenTIM) ישירות (ולא רק דרך תיעוד עליו), ברור שהארכיטקטורה שהם בחרו — Rust, פיצול ל-`part.rs`/`level_file_format.rs`/`atmosphere.rs`, ו-`nannou` לרינדור — מתאימה במדויק לצרכים של ChainWorks (חלוקת parts/level/sim זהה למה שכבר תוכנן ב-v1.0/v1.1), ו-`rapier2d` הוא ממילא Rust-crate מקורי — אין סיבה לעטוף אותו ב-WASM/JS כשאפשר להשתמש בו ישירות. **המחיר:** מאבדים את "פתח בדפדפן בטלפון תוך 30 שניות" מ-v1.1, ואת המסלול המיידי ל-App Store (Capacitor). זו החלטה מודעת: דסקטופ קודם (כמו OpenTIM עצמו), מובייל בשלב נפרד — ראה §7.5 לתוכנית המלאה כשמגיע הזמן.

**⚠️ להזכיר שוב: זו נאמנות ארכיטקטונית, לא תלות קוד.** ChainWorks לא מוסיף את ה-crate של OpenTIM כ-dependency, לא מעתיק קבצים מהם, ולא forkי את ה-repo שלהם. כל קובץ בעץ למטה נכתב מאפס. ראה §2.1 להסבר המלא על סיבת ההפרדה (רישוי GPL-3.0).

#### 5.5.2 מבנה תיקיות

```
chainworks/
├── Cargo.toml
├── schemas/
│   └── level.schema.json          # עדיין תקף כתיעוד/ולידציה — JSON Schema הוא שפה-אגנוסטי
├── data/
│   └── parts/*.json               # קטלוג הרכיבים (52 קבצים) — נטען דרך serde_json
├── levels/
│   ├── A/*.json  B/*.json  C/*.json  D/*.json
├── assets/
│   ├── sprites/  icons/  audio/  fonts/
├── src/
│   ├── main.rs                    # App::new().add_plugins(...).run() — M0/M1 (ראה src/main.rs בפועל)
│   ├── math.rs                    # PIXELS_PER_METER, Vector helpers, camera transforms
│   ├── render.rs                  # מערכות רינדור מותאמות (מעבר לספרייט האוטומטי של Bevy):
│   │                              #   EnergyFlowRenderer (חלקיקי אנרגיה, סעיף 3.9.2), Juice.
│   │                              #   מיקום/סיבוב בסיסי מתעדכן אוטומטית ע"י Bevy מ-Transform —
│   │                              #   אין צורך ב-view function ידנית כמו ב-nannou.
│   ├── debug.rs                   # שכבת דיבוג-אוברליי (כמו OpenTIM `debug.rs`)
│   ├── atmosphere.rs              # FieldSystem + ThermalSystem: רוח/ואקום/מגנט/חום/אש
│   │                              #   (כמו OpenTIM `atmosphere.rs` — אותה אחריות, מימוש חדש)
│   ├── energy_graph.rs            # EnergyGraph כ-Bevy Resource: צמתים/קשתות/propagate —
│   │                              #   "הסוד של הז'אנר" מסעיף 1.1. אין ל-OpenTIM מקבילה בשם
│   │                              #   הזה; זו תוספת ייחודית ל-ChainWorks
│   ├── rope_network.rs            # RopeNetwork: אילוצי מתיחה + ניתוב דרך גלגלות
│   ├── gear_train.rs              # GearTrain: צימוד גלגלים, יחסים, רצועות
│   ├── part.rs                    # PartDef כנתוני-Component, Port/Anchor/EnergySignal,
│   │                              #   marker components לכל קטגוריית רכיב (כמו OpenTIM `part.rs`,
│   │                              #   אך ECS: ראה §5.6 — behaviors הם Systems, לא טרייט אחד)
│   ├── parts/                     # מימוש per-part כ-Systems (כמו OpenTIM `parts/`)
│   │   ├── mod.rs                 # PartRegistry — טוען data/parts/*.json, plugin שרושם Systems
│   │   ├── fuse_cord.rs  charge_barrel.rs  punch_arm.rs
│   │   └── conveyor.rs  walker_unit.rs  balloon.rs  ...
│   ├── level_file_format.rs       # מבני serde התואמים ל-level.schema.json
│   │                              #   (כמו OpenTIM `level_file_format.rs`)
│   ├── level_load.rs              # LevelLoader: JSON → EditorState → spawn entities בעולם Bevy
│   │                              #   (כמו OpenTIM `level_load.rs`)
│   ├── editor_state.rs            # EditorState כ-Bevy Resource — המקור היחיד לאמת (סעיף 2.5)
│   ├── win_conditions.rs          # WinCondition/FailCondition + evaluate() (סעיף 2.7)
│   ├── sim/
│   │   ├── mod.rs                 # SimPlugin — קושר את כל ה-Systems לסדר הקבוע בסעיף 2.4,
│   │   │                          #   בשלב הפיזיקה של bevy_rapier2d (PhysicsSet)
│   │   ├── body_factory.rs        # PartDef → (RigidBody, Collider) components של bevy_rapier2d
│   │   ├── collision_router.rs    # CollisionEvent (bevy_rapier2d) → אירועי IMPACT
│   │   └── collision_groups.rs    # ביטמסקות (סעיף 5.7)
│   ├── input.rs                   # מקלדת/עכבר לדסקטופ — drag/drop, snap, rope tool (M5).
│   │                              #   ⚠️ ה-UX המתועד בסעיף 3 נכתב למגע/נייד; ב-M5 יש להתאים
│   │                              #   אותו לעכבר+מקלדת (קליק=touch, גרירה=drag, גלגלת=zoom)
│   │                              #   ולתעד את ההתאמה — ראה §7.5.
│   └── ui/
│       └── tokens.rs              # MOTION / ELEVATION / RADIUS / SPACE כקבועי Rust (§3.9.1)
│                                  #   HUD/מסכים מצוירים ישירות עם nannou draw (M6),
│                                  #   בלי מסגרת UI חיצונית.
└── tests/
    ├── determinism.rs      # מצב זהה אחרי 600 ticks, 3 ריצות
    ├── solutions.rs        # כל solutions[] בכל שלב עוברים (הרצה headless דרך ReplayRunner)
    ├── schema.rs           # כל קובץ שלב עובר deserialize תקין מול level_file_format
    └── unsolvable.rs       # הצבות אקראיות לא פותרות
```

### 5.6 חוזי הליבה (v1.3 — Bevy ECS: Components + Systems + Resources)

הפער העיקרי מ-v1.2: אין יותר `trait PartBehavior` עם virtual dispatch. ב-ECS, "מה רכיב עושה" מבוטא כ-**Systems שרצים על queries** של רכיבים (components) ספציפיים — זה בדיוק אותו רעיון של "רכיב לא יודע על רכיב אחר, רק פולט/צורך אנרגיה" מסעיף 1.2, רק בסינטקס טבעי יותר ל-Rust/ECS:

```rust
// src/part.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum Tier { P0, P1, P2 }

/// המידע הסטטי על טיפוס רכיב — נטען מ-data/parts/*.json, לא הרכיב-instance עצמו.
#[derive(Debug, Clone, serde::Deserialize, bevy::prelude::Resource)]
pub struct PartDef {
    pub part_type: String,
    pub display_key: String,
    pub tier: Tier,
    pub tags: Vec<String>,
    pub wind_factor: Option<f32>,
    pub ports: Vec<Port>,
    pub anchors: Vec<Anchor>,
    pub editor: EditorSpec,
    pub params: std::collections::BTreeMap<String, ParamSpec>, // BTreeMap: סדר איטרציה קבוע!
}

/// Marker components — כל entity שהוא רכיב מקבל את אלה לפי הצורך.
/// System של רכיב ספציפי (למשל fuse_cord.rs) שואל query על ה-marker שלו,
/// לא על "כל הרכיבים" — זה מה שמחליף את ה-`on_tick`/`on_energy` הישן.
#[derive(bevy::prelude::Component)]
pub struct PartInstance { pub part_type: &'static str, pub id: String }

#[derive(bevy::prelude::Component)]
pub struct EnergyPorts { pub ports: Vec<Port> }

// src/parts/fuse_cord.rs — דוגמה: כך נראה "behavior" של רכיב ב-ECS
#[derive(bevy::prelude::Component)]
pub struct FuseCord { pub burn_progress: f32, pub length: f32 }

/// מקביל ל-onEnergy(THERMAL) + onTick של v1.1/v1.2, כ-System רגיל.
/// נרשם דרך SimPlugin (sim/mod.rs) בסדר הקבוע של סעיף 2.4.
pub fn fuse_cord_burn_system(
    time: Res<Time<Fixed>>,
    mut fuses: Query<(&mut FuseCord, &EnergyPorts, &Transform)>,
    mut energy: ResMut<EnergyGraph>,
) {
    for (mut fuse, ports, transform) in &mut fuses {
        if energy.read_thermal_in(&ports) {
            fuse.burn_progress += 45.0 * time.delta_secs();
            // ... פליטת THERMAL זמני ברדיוס 8 בנקודת הבעירה, ראה §1.4.3
        }
    }
}

// src/energy_graph.rs
#[derive(bevy::prelude::Resource, Default)]
pub struct EnergyGraph { /* צמתים, קשתות — מפתח: String (part id), לא Entity,
                            כדי שה-JSON של solutions[] (§5.2) יישאר קריא/דטרמיניסטי */ }

impl EnergyGraph {
    pub fn connect(&mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) { /* ... */ }
    pub fn emit(&mut self, node_id: &str, port: &str, signal: EnergySignal) { /* ... */ }
    pub fn read(&self, node_id: &str, port: &str) -> Option<EnergySignal> { /* ... */ }
    /// מיון טופולוגי; מעגלים נפתרים ב-2 מעברים max. רץ כ-System, ראשון בסדר של §2.4.
    pub fn propagate(&mut self) { /* ... */ }
}
```

**כלל לסוכן הקוד:** כל רכיב חדש (M2 ואילך) = (1) marker component ב-`part.rs` או בקובץ הרכיב עצמו, (2) system אחד או יותר שנרשמים ל-`SimPlugin`, (3) הגדרת data ב-`data/parts/<part_type>.json`. אל תיצור trait-object registry גנרי — זה בדיוק מה ש-ECS נועד לייתר.

### 5.7 קבוצות התנגשות

```rust
// src/sim/collision_groups.rs
use rapier2d::geometry::Group;

pub const STATIC_GEOMETRY: Group = Group::GROUP_1;
pub const DYNAMIC_BODY: Group    = Group::GROUP_2;
pub const MECHANISM: Group       = Group::GROUP_3;
pub const SENSOR_ZONE: Group     = Group::GROUP_4;
pub const WALKER: Group          = Group::GROUP_5;
pub const PREVIEW_GHOST: Group   = Group::GROUP_6; // לא מתנגש בכלום, לתצוגה בלבד

// SENSOR_ZONE: collider עם sensor(true), מתנגש עם DYNAMIC_BODY | WALKER בלבד
// PREVIEW_GHOST: אף פעם לא נכנס לעולם הפיזיקה (לא מוכנס ל-ColliderSet) — בדיקת חפיפה ידנית
```

### 5.8 תוכנית מימוש — Milestones למימוש מול Claude Code

הזן את השלבים בזה אחר זה. אל תזין את כל המסמך בבת אחת — זה מוביל לקוד רדוד.

| # | Milestone | תוצר | קריטריון קבלה |
|---|---|---|---|
| **M0** | **Bootstrap** | **`bevy` + `bevy_rapier2d` נטענים, חלון ריק** | **קובייה נופלת על רצפה ב-55–60fps — ✅ הושלם (ראה `src/main.rs`)** |
| M1 | Sim core | `sim::SimPlugin`, סדר Systems קבוע (§2.4), `body_factory.rs`, טסט דטרמיניזם | מצב זהה אחרי 600 ticks, 3 ריצות |
| **M2** | **Part registry + P0 static/dynamic** | **12 רכיבים ראשונים מ-data JSON (`part.rs`, `parts/mod.rs`)** | **כדור מתגלגל על קרש — ✅ הושלם** |
| **M3** | **Level loader + win conditions** | **`level_file_format.rs` (serde), `lvl_a01_first_roll`, מכונת מצבים, reset** | **שלב 1 ניתן לפתירה ולאיפוס אינסופי — ✅ הושלם (3 סבבי פתרון+איפוס מאומתים בטסט)** |
| **M3.5** | **בדיקת אנדרואיד ראשונה (חדש ב-1.3)** | `cargo apk`/`xbuild` — לוקחים בדיוק את מה שיש מ-M3 ובונים APK debug | **רץ בפועל על הטלפון של המפתח** — רואים את השלב, גם בלי קלט מגע עדיין. זו הבדיקה הכי חשובה במסמך: לגלות בעיות Android מוקדם, לא בסוף — **⏳ CI מאומת ומייצר APK debug אמיתי (ראה M7 — `android.yml` ירוק, ארטיפקט `chainworks-debug-apk` הורד בפועל), אבל התקנה/הרצה על מכשיר אנדרואיד פיזי עדיין לא בוצעה מהסביבה הזו** (ראה הערה למטה) |
| **M4** | **Energy graph + חבלים וגלגלים** | **`energy_graph.rs`, `rope_network.rs`, `gear_train.rs`, `outlet_power`/`motor_electric`/`gear_small`/`gear_large`/`conveyor`/`pulley_wheel`** | **מנוע→גלגל→מסוע עובד, גלגלת מרימה משקולת — ✅ הושלם (`tests/gear_and_pulley.rs`: שני התרחישים נפתרים ודטרמיניסטיים ב-3 ריצות)** |
| **M5** | **קלט: דסקטופ + מגע יחד** | **`input.rs`: `PointerState` אחד לשני הקלטים (`update_pointer_from_mouse`/`update_pointer_from_touch`), גרירה מהארגז + הזזת רכיב קיים, Grid snap (§3.3 עדיפות 4)** | **גרירה מהארגז ופתרון שלם מאומתים אוטומטית (`tests/input.rs`) — כולל דרך `Touches` (אותו קוד קלט בדיוק כמו עכבר) ודרך גרירת עכבר אמיתית עם `xdotool` תחת Xvfb. ✅ הושלם, בהיקף הזה: סיבוב/היפוך/מחיקה/inspector (§3.4), כלי חיבור (§3.5) ו-pan/zoom (§3.6) נדחו במפורש ל-M6+ — מגע אמיתי על מכשיר אנדרואיד אמיתי עדיין לא נבדק (M7)** |
| **M6** | **UI shell + Design system** | `ui/tokens.rs`, PartsBin, Inspector, HUD, LevelSelect, SpeedSlider — Bevy UI/Sprite | זרימה מלאה: תפריט→שלב→פתרון→שלב הבא, בהתאמה מלאה ל-3.9, על דסקטופ **ואנדרואיד** — **⏳ חלקי: HUD (כותרת+מטרה), PartsBin אמיתי (Bevy UI, לא placeholder), כפתור ראשי Play/Stop/Reset אמיתי, ופונט (Rubik, OFL) מומשו ואומתו חזותית (`xdotool` תחת Xvfb, גרירה+לחיצה אמיתיות עד Solved). **שפת התוכן במשחק היא אנגלית** (המסמך הזה עצמו נשאר בעברית — זו רק תיעוד פנימי). `ui/tokens.rs` מכיל SPACE/RADIUS/צבעי-אנרגיה/פלטה — MOTION עדיין לא מחובר למערכת tween. Inspector, LevelSelect, SpeedSlider — טרם מומשו** |
| M6.5 | **Visual polish pass (v1.1)** | render systems לחלקיקי אנרגיה צבועים (§3.9.2), Juice מלא (טבלת 3.9.6), haptics אנדרואיד | וידאו של שלב שנפתר "מרגיש" כמו טריילר של משחק, לא פרוטוטייפ — **טרם התחיל** |
| **M7** | **אריזת דסקטופ + אנדרואיד + CI** | **GitHub Actions matrix: דסקטופ (`ubuntu`/`macos`/`windows-latest`, `cargo build --release` + `cargo test`) `.github/workflows/desktop.yml`, **וגם** `.github/workflows/android.yml` לבניית APK debug** | **⏳ חלקי: דסקטופ CI ✅ הושלם ומאומת ירוק בפועל בשלושת המערכות (`mcp` GitHub Actions — לא רק "קיים", בפועל `conclusion: success` בכל 3). אנדרואיד CI ✅ מאומת ירוק אחרי 4 באגים אמיתיים שנמצאו ותוקנו ברצף (פלטפורמה חסרה → אזל שטח דיסק → ניקוי-דיסק ששיבש את התקנת ה-NDK → `cargo apk` לא הצליח להבחין בין `[[bin]]` ל-`[lib] cdylib` עם אותו שם) — ה-run הירוק האחרון (`1b11fe3`) בנה, חתם והעלה ארטיפקט APK debug אמיתי בגודל 68MB. **עדיין חסר**: זה APK **debug** בלבד (לא release-signed כנדרש בקריטריון), ואין עדיין אימות בפועל על טלפון פיזי (M3.5 עדיין פתוח מהסיבה הזו) — release signing הוא M10/M11** |
| **M8** | **תוכן** | כל 60 שלבים + P0/P1 מלאים + כל `solutions[]` golden tests עוברים ב-CI | **⏳ חלקי: חטיבה A (A_FOUNDATIONS, שלבים 1–12 לפי §5.8's 12+16+16+16=60) הושלמה — 12/12 שלבים ממומשים ומאומתים (`tests/content_levels.rs` + `tests/gear_and_pulley.rs`; התיאור הקודם "9/10" כאן היה מונה שגוי/מיושן — 13–28 שמור ל-B, ולכן A חייבת להיות 1–12, לא 1–10). כולל שלוש תת-מערכות חדשות: A01 (נפילה חופשית), A02 (`springboard`), A04 (`lever_seesaw`), **A05 (`atmosphere.rs`'s `FieldSystem` חדש — שדה PNEUMATIC מלבני מ-`fan_blower`, דוחף כל גוף דינמי לפי `windFactor`/מסה הפוכה)**, **A06 (`switch_plate` חדש — IMPACT דרך `CollidingEntities` קיים, `SimSet::CollisionRouting`, ELECTRIC out ל-`EnergyGraph`)**, A07 (גלגלי שיניים+מסוע), **A08 (`atmosphere.rs`'s `ThermalSystem` חדש — `candle`/`fuse_cord`/`charge_barrel`: הצתה מבוססת-קרבה, התקדמות בעירה קבועה, פיצוץ רדיאלי + הריסת `DESTRUCTIBLE`)**, **A09 (`sim/collision_router.rs` חדש — כלל תגית גנרי SHARP↔POPPABLE, ו-`cutter_shears` חדש שחותך `RopeConnection` וקרבה)**, **A10 (`punch_arm` חדש — אימפולס קדימה על מגע חדש דרך אותו `CollisionRouting`)**. A03 קיים כשלב-פיתוח (`lvl_a03_pulley_lift`, לא לפי המספור המקורי). **A11 "Handle With Care" (`ball_glass`/FRAGILE כ-SUBJECT בפני עצמו לראשונה — עד כה FRAGILE הופיע רק כמכשול ב-B03; כאן המטרה להוכיח שגלגול עדין דרך רמפה רגילה לא חוצה את סף השבירה, מהירות שיא נמדדה ~344 יח'/שנ' מול הסף 576)**, **A12 "Clear the Air" (`spike_pin`/SHARP כ-פותר-מכוון בפני עצמו לראשונה — בלון חוסם שוקע/עף אל מסמר ונפוץ בתוך כ-30 טיקים, הרבה לפני שהכדור המתגלגל לאט מגיע לאזור, מפנה את הדלי; בניגוד ל-C05 שבו הכלל הזה משמש כאיום למנוע, כאן הוא מנוע-פתרון מכוון)**. באג אמיתי שנמצא ותוקן באגב: `PartTags` (`level_load.rs`) כלל בעבר רק תגיות per-instance מה-level JSON, לא את תגיות ה-part-type המובנות (`DESTRUCTIBLE`/`FLAMMABLE`/`POPPABLE`/`SHARP` מ-`data/parts/*.json`) — כלל SHARP↔POPPABLE ופיצוץ-הורס-DESTRUCTIBLE היו no-op שקט לפני התיקון. חטיבת B (B_COMBOS, שלבים 13–28 לפי §5.8): 9/16 שלבים ממומשים ומאומתים (`tests/content_levels_b.rs`): B01 "Cut Loose", B02 "Relay", B03 "Break Through" (כלל תגית FRAGILE חדש), B04 "Fling the Trigger", B05 "Windswept" (ראו למעלה), **B06 "Three in a Row" (שרשרת 3 גלגלים מותאמים — motor→gear1→gear2→gear3→מסוע. באג אמיתי שנמצא: כל צימוד גלגלים נוסף הופך את כיוון ההנעה, כך ששרשרת של 3 גלגלים צריכה כיוון מנוע הפוך משרשרת של 2 כדי שהמסוע יצא לאותו כיוון בפועל — נתפס כי הכדור פשוט נסע לכיוון ההפוך ונפל מהעולם)**, **B07 "Bounce Relay" (`springboard` (A02) משגר כדור-הדק בקשת אווירית אל `switch_plate` — השימוש הראשון בשילוב הזה; הלוח ממוקם לא לפי ניחוש אלא לפי מסלול הקשת שנמדד בפועל דרך רתמת ה-debug — לחיצה מלאה במגע אווירי חד-פעמי מספיקה כי המצב הוא toggle, ואז מנוע/גלגלים/מסוע (B01) מעבירים כדור-נושא נפרד לדלי; נפתר בניסיון התכנון השני אחרי שהניסיון הראשון מיקם את הלוח גבוה מדי מעל קודקוד הקשת בפועל)**, **B08 "Cut and Catapult" (`lever_seesaw` (A04, וכבר גם ב-B04) הפעם משולב עם חבל+גלגלת+`cutter_shears` (A09) — שילוב חדש, לא ה-lever הראשון בחטיבה B: משקולת תלויה מוחזקת ע"י חבל מתוח דרך גלגלת עד עוגן שוכב על הקרקע; חיתוך החבל משחרר אותה בדיוק כמו נפילת המשקולת המקורית של A04 — משיגה תזמון שחרור מיידי (elec_in של החותך מחובר ישירות ל-`outlet_power`, ללא כדור-הדק מפיל) כדי לא לאבד את "מרוץ הזמן" מול הנטייה הטבעית של הכדור-נושא להחליק מהמוט לפני שהמשקולת מגיעה. שני ניסיונות עיצוב אמיתיים שנכשלו ותועדו כלקח: (1) רוח (`fan_blower`) שאמורה לסטות ארגז נופל כבד מעל מרחק גדול לכיוון מוט — נכשל כי נוסחת הדחיפה (`accel = power * windFactor * (1/mass) * falloff`) חלשה מכדי להזיז גוף כבד שנופל מהר על פני מאות יחידות בזמן המוגבל שהוא שוהה בתוך השדה; (2) `punch_arm` שמשגר משקולת אנכית מעלה ישירות מעל המוט — נכשל כי הדחיפה הראשונית החלשה-יחסית איפשרה למוט להתחיל להטות והכדור-נושא להחליק ממנו כמעט מיד, עוד לפני שהמשקולת המשוגרת השלימה את מעופה וחזרה ליפול. נפתר רק בגרסה השלישית, שחזרה במדויק לפיזיקת ה-drop המוכחת של A04 והוסיפה רק את מנגנון ה"החזקה עד לחיתוך")**, **B09 "Blow the Switch" (`fan_blower` דוחף כדור-הדק קל (`ball_rubber`, מסה 0.6) שמתגלגל על הקרקע (לא נופל באוויר כמו בניסיון הרוח שנכשל ב-B08) עד ללוח לחיצה — שימוש ראשון ברוח כמנגנון שמפעיל `switch_plate` במקום מגע-כבידה רגיל. עובד בזכות חשיפה ממושכת לשדה תוך כדי גלילה נתמכת-קרקע, בניגוד לניסיון הכושל הקודם של הזזת ארגז כבד שנופל באוויר על פני מרחק גדול)**. תיקון תחזוקה אמיתי שנמצא באגב (במהלך B07): `src/level_catalog.rs`'s `ALL_LEVELS` (רשימת השלבים שהמשחק בפועל מאפשר לשחקן להגיע אליהם, דסקטופ ואנדרואיד כאחד) לא כלל את D01 כלל — נבנה ונבדק בהצלחה אך היה בלתי-נגיש בפועל דרך המשחק עד לתיקון הזה. חטיבת C (C_TIMING, שלבים 29–44): 6/16 שלבים — C01 "Slow Fuse" (כלל THERMAL×TENSION חדש ב-`atmosphere.rs`/`rope_network.rs`: חבל בטווח מקור חום נשרף ונחתך, בדיוק כמו §1.4.1 המתועד אך מעולם לא מומש. גם באג פיזיקה אמיתי: כדור שנופל אנכית לחלוטין ונוחת על קרקע שטוחה עם מהירות אופקית אפס יכול לפגוע בכיוון-חיכוך ניוון (degenerate) בפותר הפיזיקה ולקבל בעיטה אופקית בלתי-צפויה — A09 נחת נקי במקרה, C01 לא; תוקן ע"י מלכודת-תפיסה שלא תלויה במזל הנחיתה), C02 "Late Arrival" (קרש קפיצה (A02) שולח כדור למרוץ מול פתיל שכבר דלוק — שילוב תזמון + קפיצה ראשון). **C03 "Clear the Way" (כלל PNEUMATIC×THERMAL חדש ב-`atmosphere.rs`'s `wind_field_system` — רוח בעוצמה ≥2 מכבה נר לצמיתות, בדיוק כמו §1.4.1 המתועד אך מעולם לא מומש. אותו מאוורר גם דוחף בלון-SUBJECT מעבר למקום שבו עמד הנר; ה-fail condition `SUBJECT_DESTROYED` נשאר בשלב במכוון — מוכיח שהנר באמת כבה ולא רק שהבלון "הצליח לחמוק", כי לו הנר נשאר דלוק, ה-FLAMMABLE tag rule הקיים כבר היה מפוצץ את הבלון)**, **C04 "Just in Time" (אותו מנגנון PNEUMATIC×THERMAL כמו C03, אך הפעם מגודר מאחורי `switch_plate` — כדור-הדק נופל על הלוח ורק אז המאוורר קם לחיים, כך שיש עיכוב אמיתי לפני שהרוח מגיעה. שני באגים גיאומטריים אמיתיים שנתפסו ותוקנו דרך רתמת ה-debug בטרם ה-commit: (1) `fan_blower` הוא גוף פיזי מוצק (לא sensor) בדיוק כמו `charge_barrel`/`cutter_shears` בעבר — בלון שהוצב חופף לגוף המאוורר נתקע פיזית על הפינה שלו במקום לעלות/לנוע חופשי; (2) שדה הרוח פועל רק לכיוון ה-+x המקומי של המאוורר (`along < 0.0` נפסל ב-`in_fan_field`) — מיקום ראשוני של הבלון "מאחורי" המאוורר הפך את הבדיקה לשקרית תמיד. גם התגלה שהבלון הבועט (buoyant climb) ממשיך לעלות אנכית הרבה אחרי שהוא חוצה את טווח ה-x של אזור המטרה, כך שמיקום goal_zone לפי הגובה הסטטי המקורי (כמו ב-C03) פספס אותו לגמרי — תוקן ע"י מדידת המסלול בפועל וקביעת גובה ה-goal_zone לפי הגובה שבו הבלון חוצה את ה-x הנכון, לא לפי ניחוש)**, **C05 "Needle Point" (שימוש ראשון בפועל ב-`spike_pin`/SHARP בתוכן — לא רק בטסט יחידה. בלון עולה אנכית ישר לתוך מסמר קבוע אלא אם המאוורר (מגודר מאחורי `switch_plate`, אותו מנגנון כמו C04) מספיק לדחוף אותו הצידה לפני שהוא מגיע לגובה המסמר; אם המאוורר לא היה מגיע בזמן, הבלון היה פוגע ונפוץ (כלל SHARP↔POPPABLE הגנרי מ-A09). נבנה ונאומת בניסיון הראשון בזכות שימוש חוזר ישיר בגיאומטריית ה-fan/switch המדויקת שכבר הוכחה ב-C04)**, **C06 "Twin Candles" (שני מופעים בלתי-תלויים לגמרי של פתיל/חבית/קיר (A08) בו-זמנית באותו שלב — בדיקה טכנית שהמנוע מטפל נכון במספר `ThermalEmitter`/`FuseCord`/`ChargeBarrel` מקבילים בלי הפרעה הדדית (רדיוס הפיצוץ של חבית1 מגיע לטווח נר2 אך לא פוגע בו כי אין לו תגית DESTRUCTIBLE, ורדיוס הפיצוץ קטן מהמרחק לקיר2/חבית2). שני הקירות נהרסים כמעט בו-זמנית סביב tick 270, הרבה לפני שהכדור המתגלגל לאט מגיע לאף אחד מהם — נבנה ונפתר בניסיון הראשון)**. חטיבת D (D_MASTER, שלבים 45–60): 4/16 שלבים — **D01 "Chain of Command" (שרשרת ארוכה: פתיל→פיצוץ הורס קיר (A08)→כדור ממשיך להתגלגל ולוחץ לוח (A06)→מנוע/גלגלים/מסוע (B01) מעביר כדור-נושא נפרד לדלי — 4 מכניקות ברצף. שיעור עיצוב אמיתי: ניסיון ראשון השתמש בדחיפת הפיצוץ (blast impulse) כדי לשגר כדור-פסולת על פני קרקע פתוחה אל לוח לחיצה מרוחק — נכשל שוב ושוב כי (א) משקולת-נגד שנחה ליד הלוח, מחוברת דרך אותו חבל/גלגלת שמחזיקה את ה-subject באוויר, "זחלה" לאט לאורך הקרקע לכיוון הלוח בגלל מתיחות החבל (התנהגות פיזיקלית אמיתית, לא באג) והגיעה ללוח לפני הפסולת המיועדת, ו-(ב) נקודת הנחיתה של הפסולת המשוגרת הייתה בלתי-צפויה וממשיכה להתכנס לאזור קרוב לאותה משקולת-זוחלת גם אחרי כיוונון עוצמת הפיצוץ. במקום להמשיך להיאבק בחישוב מסלול לא-צפוי, השלב נבנה מחדש כולו מתבניות-משנה שכבר הוכחו (הריסת קיר ע"י פתיל מ-A08, לוח-לחיצה במגע מ-A06, בלוק מנוע/גלגלים/מסוע סטנדרטי) — ונפתר בניסיון הראשון)**, **D02 "Signal Chain" (שרשרת של 5 מכניקות: פתיל→פיצוץ הורס קיר (A08)→כדור מגיע ללוח1 (A06)→לוח1 מפעיל **חשמלית** את `cutter_shears` (לא במגע — לראשונה ברמת ה-D)→חיתוך החבל משחרר משקולת תלויה שנופלת בדיוק מעל לוח2→לוח2 מפעיל מנוע/גלגלים/מסוע (B01) שמעביר כדור-נושא נפרד לדלי. באג אמיתי שנתפס ותוקן לפני ה-commit: אותה "זחילת משקולת-נגד לאורך הקרקע בגלל מתיחות חבל" שתועדה כשיעור עיצוב ב-D01 חזרה כאן בצורה מסוכנת יותר — משקולת-הנגד (שיושבת ליד הגלגלת, לא המשקולת התלויה) זחלה במקרה בדיוק עד למיקום לוח2 ולחצה אותו **בעצמה**, עוד לפני שהכדור-הדק בכלל הגיע ללוח1, מה שגרם לכל 4 המכניקות הראשונות (פתיל/פיצוץ/לוח1/חיתוך) להיות בלתי-רלוונטיות לחלוטין ולשלב "להיפתר" ב-tick 640 דרך קיצור-דרך לא-מכוון — התגלה רק כי הבדיקה בדקה גם את מצב ה-switch/rope בפועל ולא רק "האם הכדור בדלי". תוקן ע"י קיר-עצירה (`wall_brick`) בין משקולת-הנגד לגלגלת, בדיוק כמו התיקון שכבר תועד ב-D01 — לאחר התיקון השלב אכן פותר את חמשת המכניקות ברצף הנכון (Solved ב-tick 1160). לקח לתוכן עתידי: כל שילוב future של rope+pulley+counterweight חייב לכלול קיר-עצירה מונע-זחילה כברירת מחדל, לא רק כתיקון תגובתי)**, **D03 "Bounce and Cut" (שרשרת של 4 מכניקות: קרש קפיצה (B07) משגר כדור-הדק בקשת אווירית אל לוח1→לוח1 מפעיל חשמלית את `cutter_shears` (בדיוק כמו D02)→חיתוך החבל משחרר משקולת תלויה שנופלת על לוח2→לוח2 מפעיל מנוע/גלגלים/מסוע (B01). הפעם קיר-העצירה מונע-הזחילה (הלקח מ-D02) נכלל **מראש**, לא כתיקון אחרי כישלון — ואומת ישירות דרך רתמת ה-debug שהמשקולת-נגד באמת נעצרת סביב x=420 הרבה לפני שלוח2 מופעל (ב-tick 400) ע"י המשקולת התלויה שנופלת, לא ע"י זחילה. נבנה ונפתר בניסיון הראשון (Solved ב-tick 680) בזכות שימוש חוזר מדויק בגיאומטריה המוכחת של B07 (קרש+לוח) ו-D02 (חבל+גלגלת+חותך+לוח+מנוע))**, **D04 "Wind and Fire" (שרשרת של 6 מכניקות — הראשונה בחטיבה D שמשלבת רוח: פתיל→פיצוץ הורס קיר (A08)→כדור מגיע ללוח1 (A06)→לוח1 מפעיל מאוורר (A05)→הרוח מגלגלת כדור-הדק קל שני על הקרקע ללוח2 (B09)→לוח2 מפעיל מנוע/גלגלים/מסוע (B01) שמעביר כדור-נושא נפרד לדלי. נבנה ונפתר בניסיון הראשון (Solved ב-tick 1520) בזכות הרכבה ישירה משלושה תבני-משנה מוכחים בנפרד — A08 (פתיל/קיר), A06 (לוח-מגע), ו-B09 (רוח מגלגלת כדור ללוח))** |
| M9 | Sandbox | מצב חופשי, שמירה, שיתוף שלבים | משתמש יוצר ומשתף שלב |
| M10 | Release readiness | עמוד itch.io/Steam לדסקטופ; Data Safety form + נכסי Play Console לאנדרואיד | דף הורדה ציבורי חי + APK ב-Play Console internal track |
| M11 | Soft launch | דסקטופ: itch.io/GitHub Releases. אנדרואיד: internal testing track → production בהדרגה | מעקב crash rate/retention בשתי הפלטפורמות |

**כלל מעשי (מוחלף/מוקדם ב-v1.3):** M3.5 הוא לא "טעם טוב" — הוא באמת החלק הכי חשוב בתוכנית. עד שלא רואים APK רץ בפועל על מכשיר אנדרואיד אמיתי, אי אפשר לדעת אם יש בעיות ספציפיות (הרשאות, ביצועים על GPU מובייל, גודל מסך) — ועדיף לגלות את זה עם שלב אחד ולא 60. גם ב-M1–M2, הרץ CI matrix על כל מערכות ההפעלה (לא רק Linux) לתפוס בעיות פלטפורמה מוקדם.

### 5.9 תקציב ביצועים

| מדד | יעד | תקרה |
|---|---|---|
| FPS (מחשב נייד בינוני, ~2020) | 60 | לא יורד מ-50 |
| גופים דינמיים פעילים | ≤ 120 | 250 |
| זמן `physics.step()` | < 3ms | 6ms |
| זמן `energy_graph.propagate()` | < 0.5ms | 1.5ms |
| זמן טעינת שלב | < 200ms | 400ms |
| זיכרון | — | 280MB |
| גודל בינארי (release, לפלטפורמה) | < 25MB | 45MB |

---

## 6. הנחיות לעבודה מול Claude Code

**הפרומפט הפותח המומלץ:**

> אנחנו בונים משחק פאזל פיזיקלי בשם ChainWorks, ב-Rust (nannou + rapier2d). המפרט המלא נמצא ב-`docs/GDD.md` — קרא אותו לפני שאתה כותב שורת קוד.
>
> אנחנו עובדים לפי Milestones. אתה מממש רק את ה-Milestone המבוקש ועוצר. אל תיגע בשום דבר מעבר לזה.

**כללי ברזל לכל הפרויקט (מעודכן v1.2):**

1. קומפילציה נקייה (`cargo build`), אפס `unwrap()`/`expect()` לא-מוצדק מחוץ ל-`main`/בדיקות.
2. אפס רנדומליות (`rand::random()` וכו') ואפס `Instant::now()`/`SystemTime::now()` בתוך `sim/`, `energy_graph.rs`, `rope_network.rs`, `gear_train.rs`.
3. כל רכיב מוגדר ב-data JSON (`data/parts/*.json`), לא ב-hard-code (מ-M2 ואילך; M0/M1 הם smoke test מוצהר).
4. `EditorState` הוא read-only במצב `Running`.
5. אחרי כל Milestone: הרץ `cargo build && cargo test` והראה לי שהכול ירוק.
6. **(v1.1)** כל ערך אנימציה/צל/רדיוס בממשק דרך `ui/tokens.rs` בלבד — אפס ערכים מומצאים inline.
7. **(v1.1)** אף שם, דמות, גרפיקה, פריסת שלב או טקסט שיווקי לא "בהשראת" משחק קיים באופן מזוהה (ראה סעיף 4). מכניקה ויחסים פיזיקליים — כן, קרוב ככל האפשר (ראה עיקרון-העל בסעיף 1.1).
8. **(v1.2)** מבנה המודולים בהשראת OpenTIM (`part.rs`, `level_file_format.rs`, `atmosphere.rs` וכו', ראה §5.5) — **אך אפס תלות/fork בקוד ה-GPL-3.0 שלהם עצמו.** כל קובץ נכתב מאפס (ראה §2.1).

**הרגלים שישתלמו:**

- בקש `CLAUDE.md` בשורש הפרויקט עם כללי הברזל — הסוכן קורא אותו אוטומטית בכל סשן (כבר קיים בפרויקט).
- בסוף כל Milestone, בקש commit נפרד עם תיאור ברור.
- כשמשהו נשבר בפיזיקה, בקש קודם טסט שמשחזר את הבאג, ורק אחר כך תיקון.

---

## 7. אריזה והוצאה — דסקטופ ואנדרואיד יחד (Release Pipeline, v1.3)

> **עודכן ב-v1.3:** עם המעבר ל-Bevy (§2.1a), אנדרואיד **אינו** שלב 2 נפרד יותר — הוא נבנה ונבדק במקביל לדסקטופ החל מ-M3.5. סעיף 7.5 שונה מהותית לעומת v1.2: הוא כבר לא "מה יקרה כשנגיע לשם", אלא ההוראות הקונקרטיות לבניית APK כבר עכשיו.

### 7.1 מה יוצא מ-`cargo build --release`

בניגוד לסטאק ה-Capacitor של v1.1 (שהיה צריך לעטוף web build), Rust+Bevy מייצר **בינארי נייטיבי אמיתי** לכל פלטפורמה — אין WebView, אין שכבת עטיפה, אין גוטצ'ות WASM/MIME-type. `cargo build --release --target <triple>` נותן:

| פלטפורמה | Target triple | תוצר |
|---|---|---|
| Windows | `x86_64-pc-windows-msvc` | `chainworks.exe` |
| macOS (Intel) | `x86_64-apple-darwin` | בינארי Mach-O, עוטפים ב-`.app`/`.dmg` |
| macOS (Apple Silicon) | `aarch64-apple-darwin` | כנ"ל |
| Linux | `x86_64-unknown-linux-gnu` | בינארי ELF, עוטפים ב-`.AppImage` (נייד בין הפצות) |

כלי מומלץ לאריזה נוחה חוצה-פלטפורמות: [`cargo-dist`](https://github.com/axodotdev/cargo-dist) — בונה את המטריצה, יוצר arciwes/installers, ומפרסם ל-GitHub Releases אוטומטית מ-CI. אלטרנטיבה פשוטה יותר בהתחלה: zip את ה-binary + `assets/`/`data/`/`levels/` יחד ידנית בכל job.

### 7.2 בנייה ב-CI — הצינור המלא

**GitHub Actions matrix**, שלוש מערכות הפעלה במקביל:

```yaml
# .github/workflows/release.yml (תוכן לדוגמה, ראה M7)
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: macos-latest
        target: aarch64-apple-darwin
      - os: windows-latest
        target: x86_64-pc-windows-msvc
runs-on: ${{ matrix.os }}
steps:
  - uses: actions/checkout@v4
  - uses: dtolnay/rust-toolchain@stable
    with: { targets: ${{ matrix.target }} }
  # לינוקס בלבד: apt-get install libxkbcommon-dev libwayland-dev libudev-dev
  #   libasound2-dev libxi-dev libgl1-mesa-dev libvulkan-dev (ראה תלויות M0)
  - run: cargo build --release --target ${{ matrix.target }}
  - run: cargo test --release --target ${{ matrix.target }}
  # ואז: אריזה + העלאה כ-artifact / GitHub Release
```

מרגע ה-push ועד לבינארי מוכן בשלוש מערכות ההפעלה: כ-10–15 דקות (macOS/Windows runners איטיים יותר מ-Linux).

**הפצה בפועל (M10/M11):** [itch.io](https://itch.io) הוא הערוץ הכי חסר-חיכוך למשחק דסקטופ אינדי חדש — אין תהליך אישור, אפשר לדחוף build חדש דרך `butler` (כלי ה-CLI הרשמי של itch) ישירות מ-CI. Steam הוא אופציה בהמשך (עמלת $100 לפעם אחת, תהליך review, אבל חשיפה גדולה משמעותית) — לא חובה לגרסה ראשונה.

### 7.3 עמידה בסטנדרטים בסיסיים

אין כאן חנות עם כללי סקירה נוקשים כמו Apple/Google (זה ההבדל המרכזי מ-v1.1) — אבל עדיין:

- **קרדיטים/רישוי:** קובץ `LICENSE`/`THIRD-PARTY-NOTICES` שמפרט את כל ה-crates (nannou, rapier2d וכו') ואת הרישיונות שלהם (MIT/Apache-2.0 ברובם — תואם לגמרי למוצר קנייני).
- **דירוג גיל (אם מפרסמים ל-Steam בעתיד):** עם הטבלה בסעיף 4.4 (בלי נשק חם, בלי בעלי חיים נפגעים, בלי דם) התוכן מתאים לכל הגילאים ללא צורך בדירוג מיוחד.
- **מדיניות פרטיות:** אם אין איסוף נתונים בכלל (אין analytics/telemetry ב-MVP) — אפשר הצהרה פשוטה בעמוד itch.io במקום מדיניות מלאה נפרדת.

### 7.4 נכסים נדרשים לעמוד ההורדה

| נכס | itch.io | Steam (עתידי) |
|---|---|---|
| Cover image | 630×500 | 616×353 (header capsule) + עוד כמה גדלים |
| צילומי מסך | 3–8, כל רזולוציה | 5+, 1920×1080 מומלץ |
| GIF/וידאו תצוגה | מומלץ מאוד — משחק פאזל "מוכיח את עצמו" בתנועה | trailer חובה בפועל |
| תיאור | Markdown חופשי | חובה תיאור קצר + ארוך |

**טיפ:** לצילום ה-GIF/screenshots — הראה שלב באמצע פתרון עם ה-Juice וקידוד הצבע לפי אנרגיה גלוי בבירור (סעיף 3.9.2), לא מסך תפריט.

### 7.5 אנדרואיד — בנייה מוקדמת, לא שלב 2 (עודכן מהותית ב-v1.3)

Bevy תומך Android natively דרך `winit`+`wgpu` (backend Vulkan/GLES). **הליבה (`sim/`, `energy_graph.rs`, `part.rs`, `level_*`) לא משתנה כלל בין דסקטופ לאנדרואיד** — זו בדיוק הנקודה של הבחירה ב-Bevy. מה שדורש תשומת לב הוא רק שכבת ה-entry-point והאריזה.

**כלי הבנייה: `cargo-apk` (הכי פשוט) או `xbuild` (חוצה-פלטפורמות, כולל iOS בעתיד).** למתחילים עם Bevy על אנדרואיד, `cargo-apk` הוא הנתיב המתועד והפשוט ביותר:

```bash
# חד-פעמי: התקנת ה-target וה-NDK
rustup target add aarch64-linux-android
cargo install cargo-apk
# דורש Android SDK + NDK מותקנים (ANDROID_HOME / ANDROID_NDK_HOME)
```

`main.rs` צריך נקודת כניסה עם המאקרו של Bevy לאנדרואיד:

```rust
#[cfg(target_os = "android")]
#[bevy_main]
fn main() { /* אותו App::new()...run() בדיוק כמו בדסקטופ */ }

#[cfg(not(target_os = "android"))]
fn main() { /* אותו דבר */ }
```

הגדרות ה-APK (package name, permissions, target/min SDK) נכנסות ל-`[package.metadata.android]` ב-`Cargo.toml`:

```toml
[package.metadata.android]
package = "com.yourstudio.chainworks"  # reverse-DNS, לא ניתן לשינוי אחרי פרסום ל-Play!
apk_label = "ChainWorks"
target_sdk_version = 35
min_sdk_version = 26
```

בנייה מקומית לבדיקה על המכשיר של המפתח (M3.5 ואילך):

```bash
cargo apk run --release   # בונה APK, מתקין ומריץ על מכשיר מחובר ב-adb (או אמולטור)
```

> **הערת M3.5 בפועל:** סביבת הפיתוח הזו (sandbox) חסומה ל-`dl.google.com` (מדיניות רשת ארגונית, לא תקלה זמנית) ולכן לא יכולה להוריד את ה-Android NDK ולקמפל/לקשר לאנדרואיד מקומית. הפתרון: כל שינויי הקוד/`Cargo.toml` הדרושים (`[lib] crate-type = ["cdylib","rlib"]`, `[package.metadata.android]`, `#[bevy_main]` ב-`src/app.rs`) בוצעו כאן, אבל בניית ה-APK עצמה עברה ל-`.github/workflows/android.yml` — ריצה על `ubuntu-latest` עם גישה רגילה לאינטרנט. את ה-APK (debug, לא חתום לחנות) מורידים כ-artifact מה-workflow run ומתקינים עם `adb install`. **אי אפשר לאמת מכאן שה-APK באמת רץ על מכשיר אנדרואיד אמיתי** — זו בדיקה שרק המפתח יכול לבצע.

### 7.6 CI לאנדרואיד — מצטרף למטריצת הדסקטופ (M7)

```yaml
# הרחבת ה-matrix מ-§7.2 בעוד job נפרד (בונה על ubuntu-latest, לא צריך macOS/Windows)
android:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with: { targets: aarch64-linux-android }
    - uses: android-actions/setup-android@v3
    - run: cargo install cargo-apk
    - run: cargo apk build --release
    # חתימה: keystore מ-GitHub Secrets (בדיוק כמו ב-v1.1 §7 — זה לא השתנה)
    # העלאה: r0adkll/upload-google-play ל-internal testing track
```

### 7.7 הגשה בפועל ל-Google Play — זהה למה שכבר תועד

ברגע שיש APK/AAB חתום, תהליך ההגשה עצמו (Google Play Console, $25 חד-פעמי, Data Safety form, דירוג גיל, נכסי חנות — אייקון 512×512, צילומי מסך, תיאור) **זהה לחלוטין** למה שתועד בגרסאות קודמות של המסמך הזה. שום דבר בתהליך המנהלי לא השתנה — רק שכבת ה-build שמובילה לשם, וזמן ההגעה אליו (הרבה יותר מוקדם עכשיו).

**מסלול מומלץ לפני Play Store רשמי:** APK חתום debug מותקן ישירות דרך `adb install` (M3.5–M6) → internal testing track ב-Play Console (M10, בלי review ציבורי, לבדיקה עצמית) → production release (M11, אחרי soft launch מוצלח).

**iOS נשאר שלב נפרד עתידי** (דורש Mac ל-build, חשבון Apple Developer $99/שנה, ותהליך חתימה שונה לגמרי) — לא בדחיפות כרגע כי המפתח לא צריך אותו כדי לשחק בעצמו.

---

## נספח: 10 השלבים הראשונים (שלד לחטיבה A)

| # | שם | רכיב חדש | המכניקה הנלמדת |
|---|---|---|---|
| A01 | נפילה חופשית | `plank_wood` | כבידה + מדרון |
| A02 | קפיצה | `ball_rubber`, `springboard` | אלסטיות והחזרה |
| A03 | משיכה מלמעלה | `pulley_wheel`, `rope` | חבל משנה כיוון כוח |
| A04 | כוח המנוף | `lever_seesaw` | המרת נפילה לזריקה |
| A05 | רוח בגב | `fan_blower`, `outlet_power`, `balloon_lift` | שדה אוויר + כבידה שלילית |
| A06 | הלחיצה | `switch_plate` | טריגר מגע → חשמל |
| A07 | גלגלי שיניים | `gear_small`, `gear_large`, `motor_electric` | יחס העברה והיפוך כיוון |
| A08 | הפתיל | `candle`, `fuse_cord`, `charge_barrel` | השהיה מבוקרת בזמן |
| A09 | הגזירה | `cutter_shears`, `spike_pin` | ניתוק וחיתוך |
| A10 | האגרוף | `punch_arm`, `crate_wood` | אימפולס מכוון |

כל שלב ב-A מציג רכיב אחד חדש, נפתר ב-1–3 הצבות, ואורך פחות מ-90 שניות לשחקן חדש.
