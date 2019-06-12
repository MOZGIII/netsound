from dotenv import dotenv_values, find_dotenv
import lldb

env_vars = dotenv_values(find_dotenv())
env_vars = ["%s=%s" % (k, v) for k, v in env_vars.items()]

launch_info = lldb.target.GetLaunchInfo()
launch_info.SetEnvironmentEntries(env_vars, True)
lldb.target.SetLaunchInfo(launch_info)
