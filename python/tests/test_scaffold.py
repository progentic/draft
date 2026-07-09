"""Smoke tests for the Python helper-worker scaffold."""

import unittest

from draft_helpers import PACKAGE_NAME


class HelperScaffoldTest(unittest.TestCase):
    def test_package_is_importable(self) -> None:
        self.assertEqual(PACKAGE_NAME, "draft_helpers")


if __name__ == "__main__":
    unittest.main()
