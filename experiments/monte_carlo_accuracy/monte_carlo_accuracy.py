import re
from math import sqrt

from matplotlib import pyplot as plt

qn = "53n2"
old = False

with open(f"{qn}_montecarlo{'_old' if old else ''}.txt") as fil:
    monte_carlo_dump = fil.readlines()[-1]

with open(f"{qn}_probdistr.txt") as fil:
    prob_distribution_dump = fil.readlines()[-1]

monte_carlo_matcher = re.compile(
    r"ComponentForm { xs: \[([^\]]+)\], ys: \[([^\]]+)\], zs: \[([^\]]+)\], vals: \[([^\]]+)\] }"
)

prob_distr_matcher = re.compile(r"\(\[([^\]]+)\], \[([^\]]+)\]\)")


def parse_list(comma_separated_floats: str) -> list[float]:
    return [float(s.strip()) for s in comma_separated_floats.split(",")]


match = monte_carlo_matcher.match(monte_carlo_dump)
assert match is not None
xyz = (parse_list(match.group(i)) for i in (1, 2, 3))
rs = [sqrt(x*x + y*y + z*z) for (x, y, z) in zip(*xyz)]  # type: ignore

match = prob_distr_matcher.match(prob_distribution_dump)
assert match is not None
prob_r = parse_list(match.group(1))
prob_v = parse_list(match.group(2))

ax = plt.axes()
ax.hist(x=rs, bins=100)
ax2 = ax.twinx()
ax2.plot(prob_r, prob_v, color="orange")
plt.xlim((0, prob_r[-1]))
plt.ylim(0)
plt.savefig(f"out_{'old_' if old else ''}{qn}.png")
