# Student missions

Student missions describe the learner journey expected by eatme scenarios. They
focus on visible Alice behavior, prediction, observation, iteration, and
reflection.

## Student mission goals

Student missions help learners:

- understand the creative goal
- predict what an Alice action or code change will do
- run the world and observe visible behavior
- compare expected and actual results
- revise one meaningful choice
- explain the final behavior in their own words
- share evidence with an instructor or peer

## Student persona crew

The canonical student persona crew lives in:

```text
assets/personas/alice-user-crew.yaml
```

Student personas include:

| Persona | Learner focus |
| --- | --- |
| `curious-novice` | Safe cause-and-effect experiments |
| `creative-storyteller` | Narrative, camera, audio, and character choices |
| `playful-tinkerer` | Surprising variations and bugs as clues |
| `systems-puzzle-solver` | Rule-based games and simulations |
| `reflective-debugger` | Expected-versus-actual repair tests |
| `collaborative-peer-mentor` | Help through questions and evidence |
| `accessibility-advocate` | Communication across audience needs |
| `vr-player-tester` | Comfort, orientation, and fallback access |
| `media-audio-creator` | Audio, camera, timing, captions, and media cues |
| `model-texture-importer` | Responsible import and fallback behavior |

## Mission rhythm

A student mission follows this rhythm:

1. **Prompt** - Read the creative or technical mission.
2. **Predict** - State what should happen before running the world.
3. **Build** - Make a small Alice scene, code, camera, audio, or behavior change.
4. **Run** - Observe the world.
5. **Compare** - Name expected versus actual behavior.
6. **Revise** - Change one meaningful thing.
7. **Reflect** - Explain what changed and why it matters.
8. **Share** - Provide the artifact, screenshot, description, or reflection
   requested by the mission.

## Evidence expectations

Good student evidence includes:

- a named Alice world or scenario
- one visible behavior that can be observed by someone else
- a prediction made before running
- an observation made after running
- one revision based on evidence
- a short explanation of cause and effect
- any required screenshot, exported file, or reflection text

For VR or hardware-dependent missions, evidence should state whether real
hardware was available. If it was not, the student should use the documented
desktop fallback instead of pretending the VR path was tested.

## Example student mission

```text
Mission: Building a Scene First World

Create a small Alice scene with at least two objects. Before running, predict
what the audience will notice first. Run the world, observe the result, revise
one placement, camera, or timing choice, then explain how the revision changed
the audience experience.
```

Expected response shape:

```text
Prediction:
I expected the viewer to notice the penguin first because it starts closest to
the camera.

Observation:
The tree blocked part of the penguin, so the viewer noticed the tree instead.

Revision:
I moved the penguin forward and rotated the camera slightly.

Reflection:
The scene now communicates the intended character focus because the first visible
movement and camera angle point toward the penguin.
```

## What missions avoid

Student missions should not require:

- exact UI coordinates
- hidden implementation details
- memorized click paths without concept evidence
- visual polish without behavior or explanation
- a single correct creative answer

The mission succeeds when the learner can show evidence of thinking and visible
Alice behavior, not when every student produces the same world.

