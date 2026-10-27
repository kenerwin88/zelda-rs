import unittest
import json
from pathlib import Path
import tempfile

import lagless_audio_notes as notes


def frame(writes, host=0):
    return dict(writes=writes, host=host, iteration=host, context=[7, 0, 1], music={})


def note(source=1, time=0, voice=0, pitch=4096):
    return dict(voice=voice, source=source, pitch=pitch, time=time,
                volume=[50, 50], adsr=[143, 31], gain=0, end=None)


class NoteTests(unittest.TestCase):
    def test_register_snapshots_signed_volume_and_subsample_phase(self):
        on, edges = notes.extract_notes([frame([
            [0, 0, 0, 255], [0, 1, 1, 64], [0, 2, 2, 52],
            [0, 3, 3, 255], [0, 4, 4, 7], [0, 5, 5, 143],
            [0, 6, 6, 31], [10, 16, 0x4c, 1], [11, 0, 0, 40],
            [20, 0, 0x5c, 1], [21, 0, 0x5c, 1]])])
        self.assertEqual(len(on), 1)
        self.assertEqual([e['kind'] for e in edges], ['on', 'off', 'off'])
        self.assertEqual(on[0]['time'], 10.5)
        self.assertEqual(on[0]['volume'], [-1, 64])
        self.assertEqual(on[0]['pitch'], 0x3f34)
        self.assertEqual(on[0]['source'], 7)
        self.assertEqual(on[0]['contour'], [[0.5, 0, 40]])
        self.assertEqual(on[0]['end'], 20)

    def test_polyphonic_mask_and_retrigger(self):
        on, edges = notes.extract_notes([frame([[1, 0, 0x4c, 5], [2, 0, 0x4c, 1]])])
        self.assertEqual([n['voice'] for n in on], [0, 2, 0])
        self.assertEqual(on[0]['end_kind'], 'retrigger')
        self.assertEqual(on[1]['end_kind'], 'capture_end')
        self.assertEqual(len(edges), 3)

    def test_missing_note_does_not_shift_later_pairs(self):
        a = [note(i) for i in (1, 2, 3, 4)]
        b = [note(i) for i in (1, 3, 4)]
        self.assertEqual(notes.align(a, b), [(0, 0, 'exact'), (1, None, 'missing'),
                                           (2, 1, 'exact'), (3, 2, 'exact')])

    def test_pitch_change_is_a_substitution_between_exact_anchors(self):
        a = [note(i) for i in (1, 2, 3)]
        b = [dict(n) for n in a]
        b[1]['pitch'] += 1
        self.assertEqual(notes.align(a, b)[1], (1, 1, 'substitution'))

    def test_song_shift_separate_from_note_spacing(self):
        a = [note(1, 100), note(2, 420), note(3, 740)]
        b = [dict(n, time=n['time'] - 50) for n in a]
        result = notes.compare(a, b, [])
        self.assertTrue(all(r['relative_first_note_ms'] == 0 for r in result))
        self.assertTrue(all(r['onset_shift_ms'] == -50 / 32 for r in result))

    def test_song_anchor_prevents_cross_song_note_matching(self):
        a = [note(1, 100), note(1, 300)]
        b = [note(1, 100)]
        result = notes.compare(a, b, [dict(original=200, lagless=200)])
        self.assertEqual([r['status'] for r in result], ['exact', 'missing'])

    def test_song_origin_does_not_hide_one_late_voice(self):
        a = [note(1, 100, voice=0), note(2, 100, voice=1)]
        b = [note(1, 50, voice=0), note(2, 370, voice=1)]
        result = notes.compare(a, b, [])
        self.assertEqual(result[0]['relative_first_note_ms'], 0)
        self.assertEqual(result[1]['relative_first_note_ms'], 10)

    def test_emitted_snapshot_verification_rejects_wrong_phase(self):
        _, edges = notes.extract_notes([frame([[10, 16, 0x4c, 1]])])
        emitted = dict(edges[0], sample=10, phase=16)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'edges.jsonl'
            path.write_text(json.dumps(emitted) + '\n')
            self.assertEqual(notes.verify_emitted_edges(path, edges), 1)
            emitted['phase'] = 15
            path.write_text(json.dumps(emitted) + '\n')
            with self.assertRaisesRegex(ValueError, 'timestamp mismatch'):
                notes.verify_emitted_edges(path, edges)


if __name__ == '__main__':
    unittest.main()
