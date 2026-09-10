# The cached comparison simulates on a worker thread

`./parity cached-av` was single-threaded apart from its ledger parser. A
timed 200,000-frame run split its 324 seconds as: game logic 26%, audio
render and hash 15%, display capture 9%, the renderer's CPU-side source
extraction 23%, compositor submit 14%, the RGB SHA-256 7%, readback and
serialization 3%. More than half is video work that needs only the
per-frame capture snapshot, not the game.

The game now runs on a scoped simulation thread: receipt installation,
the frame, the display capture, audio render and digest, WRAM dumps, the
ROM-random drift check and the paired-checkpoint clone all happen there,
and each frame's outputs cross a bounded channel (four frames deep) to
the main thread. The main thread keeps the native frontend, which owns a
winit event loop and must stay on the main thread, and does the GPU
render, the pipelined readback, the hashing, the comparison, the ledger
writes and the checkpoint files, exactly as before and in the same
order. A stopped comparison drops the receiver and the simulation thread
ends at its next send; a passing run joins it and continues with the
final game state for the WRAM endpoint dump and the paired frontier.

The renderer's capture step became `queue_capture_video_digest`, taking
an owned capture; capture time measured on the simulation thread is
added to the renderer's timing so the stage report is unchanged.

Compatibility constraints remain explicit:

- Every digest is computed from the same capture, audio buffer and
  receipts as before; the thread boundary carries values, so the results
  cannot depend on scheduling.
- Frames are compared in ledger order; the pending queue and the GPU
  readback pipeline depth are unchanged.
- The provenance checks (schema, contiguity, input, receipt host call)
  still run before each frame is simulated, now on the simulation thread.
- Errors that previously exited the process still exit it from the
  simulation thread; a panic there fails the comparison.

## Verification

The timed 200,000-frame comparison matched every video and audio hash in
193.3 seconds against 323.9 seconds for the same frames on the same
machine before the change (40% less wall time), with the same per-stage
CPU totals.

The full cold route on the threaded binary matched all 1,581,079 frames of
video and audio in 1,528 seconds (2,514 seconds for the previous route on
this machine the same afternoon), with the four WRAM goldens and the
full-route WRAM endpoint (`31619379…`) unchanged; the run is promoted in
`routes/full_run/parity-frontier.json`.
