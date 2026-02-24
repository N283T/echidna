"""ChimeraX-HelloWorld - ChimeraX HelloWorld bundle"""

from chimerax.core.toolshed import BundleAPI


class _HelloWorldAPI(BundleAPI):
    """Bundle API for ChimeraX-HelloWorld."""

    api_version = 1

    @staticmethod
    def register_command(bundle_info, command_info, logger):
        """Register the 'hello_world' command."""
        from . import cmd

        if command_info.name == "hello_world":
            from chimerax.core.commands import register

            register(
                command_info.name,
                cmd.hello_world_desc,
                cmd.hello_world,
            )


bundle_api = _HelloWorldAPI()
