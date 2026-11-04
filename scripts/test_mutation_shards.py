"""Sharding of the targeted mutation check: complete, disjoint, stable; no Rust build."""

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "mutation_check", Path(__file__).with_name("mutation-check.py")
)
mutation_check = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(mutation_check)


class ShardTests(unittest.TestCase):
    def test_every_item_lands_in_exactly_one_shard(self):
        todo = list(range(127))
        for n in (1, 2, 3, 4, 7):
            parts = [mutation_check.shard(todo, (k, n)) for k in range(1, n + 1)]
            self.assertEqual(sorted(i for part in parts for i in part), todo)
            self.assertEqual(sum(len(part) for part in parts), len(todo))

    def test_no_shard_selects_everything(self):
        todo = list(range(10))
        self.assertEqual(mutation_check.shard(todo, None), todo)

    def test_real_mutation_list_is_split_completely(self):
        todo = list(mutation_check.mutations(set()))
        self.assertTrue(todo)
        parts = [mutation_check.shard(todo, (k, 4)) for k in range(1, 5)]
        self.assertEqual(sum(len(part) for part in parts), len(todo))
        self.assertLessEqual(max(map(len, parts)) - min(map(len, parts)), 1)

    def test_malformed_shard_is_rejected(self):
        for text in ("0/4", "5/4", "a/b", "3", "1/0"):
            with self.assertRaises(Exception):
                mutation_check.parse_shard(text)


if __name__ == "__main__":
    unittest.main()
