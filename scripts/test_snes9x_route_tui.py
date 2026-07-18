import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPTS = Path(__file__).parent
sys.path.insert(0, str(SCRIPTS))
import snes9x_route_tui as TUI  # noqa: E402


class Snes9xRouteTuiTests(unittest.TestCase):
    def make_project(self, root: Path, name: str = "clean-game") -> Path:
        project = root / name
        (project / "boundaries/0000").mkdir(parents=True)
        (project / "manifest.json").write_text(
            json.dumps(
                {
                    "kind": TUI.RECORDING_KIND,
                    "identity": {"core_name": "Snes9x", "core_version": "1.63"},
                    "boundaries": [
                        {
                            "id": 0,
                            "reset_start": True,
                            "state_path": "boundaries/0000/oracle.state",
                            "sram_path": "boundaries/0000/sram.bin",
                            "screenshot_path": "boundaries/0000/frame.png",
                            "telemetry": {"main": 7, "sub": 1, "health": 24},
                        }
                    ],
                    "takes": [
                        {
                            "id": 0,
                            "start_boundary": 0,
                            "frames": 120,
                            "status": "complete",
                        }
                    ],
                }
            )
        )
        (project / "labels.json").write_text(
            json.dumps(
                {
                    "kind": "zelda3_snes9x_boundary_labels_v1",
                    "boundaries": {"0": "Uncle rescue"},
                }
            )
        )
        (project / "sram-origin.json").write_text(
            json.dumps(
                {
                    "kind": "zelda3_snes9x_sram_origin_v1",
                    "source": "blank",
                }
            )
        )
        return project

    def test_discovers_only_recorder_projects_and_loads_labels(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.make_project(root)
            invalid = root / "not-a-route"
            invalid.mkdir()
            (invalid / "manifest.json").write_text(json.dumps({"kind": "other"}))

            projects = TUI.discover_projects(root, root / "missing-default")

            self.assertEqual([item.path for item in projects], [project.resolve()])
            self.assertEqual(projects[0].labels["0"], "Uncle rescue")
            self.assertIn("blank", TUI.storage_description(projects[0]))

    def test_resume_command_pins_selected_project_and_boundary(self):
        config = TUI.TuiConfig(
            project_root=Path("routes"),
            included_project=Path("default-route"),
            binary=Path("zelda3"),
            core=Path("snes9x.dylib"),
            rom=Path("zelda3.sfc"),
            no_build=True,
            recorder_script=Path("recorder.py"),
        )

        command = TUI.build_resume_command(config, Path("routes/clean-game"), 2)

        self.assertEqual(command[1:3], ["recorder.py", "record"])
        self.assertIn("routes/clean-game", command)
        self.assertEqual(command[command.index("--start") + 1], "2")
        self.assertIn("--no-build", command)

    def test_boundary_and_take_rows_show_operator_fields(self):
        with tempfile.TemporaryDirectory() as tmp:
            project_path = self.make_project(Path(tmp))
            project = TUI.discover_projects(Path(tmp), project_path)[0]

            self.assertIn("Uncle rescue", TUI.boundary_line(project, project.boundaries[0]))
            self.assertIn("hp=24", TUI.boundary_line(project, project.boundaries[0]))
            self.assertIn("parity=reset-ready", TUI.boundary_line(project, project.boundaries[0]))
            self.assertIn("frames=120", TUI.take_line(project.takes[0]))
            self.assertIn("1 saves / 1 takes", TUI.project_line(project))

    def test_archived_projects_are_hidden_and_can_be_shown_for_restore(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            active = self.make_project(root, "active")
            archived = self.make_project(root, "archived")
            TUI.recorder.set_project_archived(archived, True)

            visible = TUI.discover_projects(root, root / "missing-default")
            with_hidden = TUI.discover_projects(
                root, root / "missing-default", show_hidden=True
            )

            self.assertEqual([item.path for item in visible], [active.resolve()])
            self.assertEqual(
                {item.path for item in with_hidden},
                {active.resolve(), archived.resolve()},
            )
            archived_view = next(item for item in with_hidden if item.path == archived.resolve())
            self.assertTrue(archived_view.archived)

    def test_project_focused_archive_does_not_archive_a_boundary(self):
        with tempfile.TemporaryDirectory() as tmp:
            project_path = self.make_project(Path(tmp))
            project = TUI.discover_projects(Path(tmp), project_path)[0]
            state = TUI.BrowserState([project], focus="projects")

            message = TUI.toggle_selected_archive(state)

            self.assertIn("Project", message)
            self.assertTrue(TUI.recorder.load_labels(project_path)["archived_project"])
            self.assertEqual(
                TUI.recorder.load_labels(project_path)["archived_boundaries"], []
            )

    def test_project_can_be_renamed_without_changing_its_recording(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.make_project(root, "clean")

            renamed = TUI.rename_project(root, project, "Uncle")

            self.assertEqual(renamed, (root / "Uncle").resolve())
            self.assertFalse(project.exists())
            self.assertTrue((renamed / "manifest.json").is_file())
            self.assertEqual(
                json.loads((renamed / "manifest.json").read_text())["takes"][0]["frames"],
                120,
            )

    def test_viewport_scrolls_to_keep_selected_save_visible(self):
        self.assertEqual(TUI.viewport_start(total=30, selected=0, rows=10), 0)
        self.assertEqual(TUI.viewport_start(total=30, selected=9, rows=10), 0)
        self.assertEqual(TUI.viewport_start(total=30, selected=10, rows=10), 1)
        self.assertEqual(TUI.viewport_start(total=30, selected=29, rows=10), 20)

    def test_browser_defaults_to_latest_boundary(self):
        with tempfile.TemporaryDirectory() as tmp:
            project_path = self.make_project(Path(tmp))
            manifest = json.loads((project_path / "manifest.json").read_text())
            manifest["boundaries"].append(
                {
                    "id": 1,
                    "state_path": "boundaries/0001/oracle.state",
                    "sram_path": "boundaries/0001/sram.bin",
                    "telemetry": {},
                }
            )
            (project_path / "manifest.json").write_text(json.dumps(manifest))
            project = TUI.discover_projects(Path(tmp), project_path)[0]

            state = TUI.BrowserState([project])

            self.assertEqual(state.item["id"], 1)

    def test_hidden_boundaries_and_takes_are_filtered_until_requested(self):
        with tempfile.TemporaryDirectory() as tmp:
            project_path = self.make_project(Path(tmp))
            labels = json.loads((project_path / "labels.json").read_text())
            labels["archived_boundaries"] = [0]
            (project_path / "labels.json").write_text(json.dumps(labels))
            manifest = json.loads((project_path / "manifest.json").read_text())
            manifest["takes"][0]["status"] = "discarded"
            (project_path / "manifest.json").write_text(json.dumps(manifest))
            project = TUI.discover_projects(Path(tmp), project_path)[0]
            state = TUI.BrowserState([project])

            self.assertEqual(state.items, [])
            state.show_hidden = True
            self.assertEqual([item["id"] for item in state.items], [0])
            state.item_mode = "takes"
            self.assertEqual([item["id"] for item in state.items], [0])


if __name__ == "__main__":
    unittest.main()
