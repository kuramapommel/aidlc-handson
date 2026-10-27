# Project-Level Rules

> Project-specific specialisation and corrections. Loaded after `org.md` and
> `team.md` as strict-additive guidance; contradictions with broader policy
> are rejected. Populated by practices-discovery and the self-learning loop.
>
> Use sparingly: most teams don't need a project layer. Reach for it
> only when this specific project needs stable, durable guidance beyond the
> team practice (for example, package-specific release checks or an additional
> regression suite for a legacy component).

## Way of Working

<!-- Project-specific specialisation. Example: -->
<!-- This monorepo requires package-scoped branch names and a package owner -->
<!-- review in addition to the team's normal merge policy. -->

## Walking Skeleton

<!-- Project-specific specialisation. Example: -->
<!-- The walking skeleton must exercise the legacy service adapter as well -->
<!-- as the new service boundary. -->

## Testing Posture

<!-- Project-specific specialisation. -->

## Guard Policy

<!-- Project-specific. Mode: strict, relaxed, or off. Strict here holds for every intent, whatever one person asks in chat; relaxed or off here applies to every intent whose policy came from its scope, unless the person sets another; changing this line changes it. A section under the retired Change Control heading, written by an earlier release, is still read. -->

## Deployment

<!-- Project-specific specialisation. -->

## Code Style

<!-- Project-specific specialisation. -->

## Tech Stack

<!-- Technology choices locked for this project. -->

## Decided

<!-- Decisions made in earlier stages that should not be re-asked. -->
<!-- Format: DECIDED: [decision] (Stage [slug], [date]) -->

## Scope Overrides

<!-- Custom scope rules for this project. -->

## Forbidden

<!-- Populated by practices-discovery affirmation gate. -->
<!-- Format: NEVER [behavior] (affirmed [date]) -->
<!-- Example: NEVER throw exceptions across service layer boundaries (affirmed 2026-05-17) -->

## Mandated

<!-- Populated by practices-discovery affirmation gate. -->
<!-- Format: ALWAYS [behavior] (affirmed [date]) -->
<!-- Example: ALWAYS use Result<T,E> for fallible operations in service layer (affirmed 2026-05-17) -->

## Corrections

<!-- Project-specific corrections from human feedback. -->
<!-- Format: NEVER/ALWAYS [behavior] (learned [date]) -->
- このプロジェクトの「EC サイト」は、契約企業の従業員が研修予算（ポイント）で研修コースの受講用 e チケットを購入するサイト（LMS 型）を指す。カートは持たず、1 回の購入で 1 つの日程を購入する。 (learned 2026-10-27) <!-- cid:261027-simple-ec-site:requirements-analysis:73f70c23613612ed94ad92d9998e638ede62fd67e40e583b37783fc403923d39 -->
- 質問への回答で前提（商材・利用者など）が変わったら、残りの質問の選択肢をその前提に合わせて書き換えてから質問する。 (learned 2026-10-27) <!-- cid:261027-simple-ec-site:requirements-analysis:93c6b877de1cb53f8c65297a4460ad1de2c791bc98c4e53161fb2f35cee1a59f -->
- 構成は六角形アーキテクチャとし、業務コンポーネントは HTTP・データ保存・画面に依存させない。Web API・データストア・Web 画面はアダプターのコンポーネントとして明示し、依存の逆転はインターフェースではなく関数の受け渡しで行う。 (learned 2026-10-27) <!-- cid:261027-simple-ec-site:domain-design:a268aec029afa248fe5bd4eaf9eb37eb6563d83a06df21ae43f04f72e713bae8 -->
- コンポーネント間の循環依存は、必要な値を返す関数を引数として受け取る形（高階関数）にして解消する。 (learned 2026-10-27) <!-- cid:261027-simple-ec-site:domain-design:ee9abea8d9f301987843da371bd813ce19a948ced070eb3f8b2f4d5f8d992cbf -->
