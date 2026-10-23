# Reference prompts (ROADMAP A7)

How well do coding agents make videos with oxframe? Every release is
measured on the same prompts, so we see whether the library gets better for
agents, not only for people.

```sh
python3 evals/run.py                         # all prompts with Claude Code
python3 evals/run.py --only chart_pie        # some of them
python3 evals/run.py --agent none            # no agent: checks the pipeline, costs nothing
python3 evals/run.py --sdk-path ../oxframe-sdk   # projects on an unreleased SDK
python3 evals/turns.py evals/results/<run>/      # where the turns went, from the saved sessions
```

For each prompt the runner creates a project with `oxframe init`, gives the
prompt to the agent (Claude Code by default: `claude -p`, edits allowed,
`--max-budget-usd 2` per prompt) and then measures:

| | |
|---|---|
| **build** | the project compiles (`oxframe check` wrote a report) |
| **check err** | errors from `oxframe check` (visual lint) |
| **expectations** | what the prompt asked for: `uses` (e.g. `Camera`; `Particles|Custom` accepts either), `aspects` (`9:16`), `min_seconds` |
| **ok** | built, no check errors, expectations met |
| **turns, min, tokens, $** | what it took (from the agent's own report) |

Results are saved to `results/<oxframe version>-<time>.json`; the runner
compares them with the previous run of the same agent and exits non-zero on
a regression (pass rate −10 %, turns or tokens +25 %, a prompt that passed
and now fails). A run of all prompts with Claude Code costs real tokens:
roughly the per-prompt budget times the number of prompts at most.

## Adding prompts

Every new feature adds one or two prompts that need it: a `.toml` file in
`prompts/` with `category`, `prompt` (written the way a user would ask) and
optional `[expect]`. A prompt for a feature that does not exist yet sets
`requires = "<feature>"` and is skipped until `AVAILABLE` in `run.py` lists
it.

When agents fail, fix the API, the docs or the hints in the agent files,
not the prompt.
