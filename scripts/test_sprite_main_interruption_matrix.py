import unittest

from sprite_main_interruption_matrix import interruption_rows


class SpriteMainInterruptionMatrixTests(unittest.TestCase):
    def test_pending_progress_and_accepted_interruption_are_distinct(self):
        receipts = [
            {"host_call": 10, "semantic": [
                {"SpriteMainProgressed": {"AfterTimersAndOam": 2}}]},
            {"host_call": 11, "semantic": [
                {"NmiAccepted": "LatchHeld"}, "NmiHandlerCompleted",
                "SpriteMainReturned"]},
            {"host_call": 12, "semantic": [
                {"NmiAccepted": "LatchHeld"},
                {"MainLoopInterrupted": {"SpriteMainAfterTimersAndOam": 3}},
                {"SpriteMainProgressed": {"AfterTimersAndOam": 3}}]},
            {"host_call": 13, "semantic": ["NmiHandlerCompleted", "SpriteMainReturned"]},
        ]
        rows = list(interruption_rows(receipts))
        self.assertEqual(len(rows), 2)
        self.assertEqual((rows[0]["acceptances"], rows[0]["next_acceptances"],
                          rows[0]["next_sprite_returned"]),
                         ([], ["LatchHeld"], True))
        self.assertEqual((rows[1]["acceptances"], rows[1]["next_acceptances"],
                          rows[1]["next_sprite_returned"]),
                         (["LatchHeld"], [], True))

    def test_preserves_both_acceptances_in_one_host(self):
        receipts = [
            {"host_call": 20, "semantic": [
                {"NmiAccepted": "Open"}, {"MainLoopProgress": "IterationStarted"},
                {"NmiAccepted": "LatchHeld"},
                {"SpriteMainProgressed": {"AfterTimersAndOam": 3}}]},
            {"host_call": 21, "semantic": ["SpriteMainReturned"]},
        ]
        self.assertEqual(list(interruption_rows(receipts))[0]["acceptances"],
                         ["Open", "LatchHeld"])


if __name__ == "__main__":
    unittest.main()
