"""Every episode's closing card names the site, from one constant."""

import pathlib
import unittest

import episode

OLD_HOST = "ndouglas.github.io/SugarScape"


class SiteUrlTest(unittest.TestCase):
    def test_site_url_is_the_renamed_repository_page(self):
        self.assertEqual(episode.SITE_URL, "ndouglas.github.io/BottomUpLab")

    def test_every_episode_closes_on_the_site_url(self):
        names = sorted(p.parent.name for p in episode.ROOT.glob("episodes/*/beats.py"))
        self.assertGreaterEqual(len(names), 28)
        for name in names:
            with self.subTest(episode=name):
                end = [b for b in episode.load_episode(name) if b.name == "end"][-1]
                self.assertTrue(end.caption.endswith(episode.SITE_URL), end.caption)

    def test_no_episode_source_names_the_old_host(self):
        offenders = []
        sources = [pathlib.Path(episode.__file__), *episode.ROOT.glob("episodes/*/*.py")]
        for path in sorted(sources):
            if OLD_HOST in path.read_text(encoding="utf-8"):
                offenders.append(str(path.relative_to(episode.ROOT)))
        self.assertEqual(offenders, [])


if __name__ == "__main__":
    unittest.main()
