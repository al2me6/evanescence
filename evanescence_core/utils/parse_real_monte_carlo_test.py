import json
import sys

from matplotlib import pyplot as plt

with open(sys.argv[1], "r", encoding="utf8") as file:
    data = json.load(file)
    rho = data["rho"]
    cdf = data["cdf"]
    data_len = len(rho)
    min_expected = cdf[0]
    max_expected = cdf[-1]
    rho_cdf = [
        (i + 1) / data_len * (max_expected - min_expected) + min_expected for i in range(data_len)
    ]
    plt.plot(rho, rho_cdf)
    plt.plot(rho, cdf)
    plt.title(f"{data['name']}, K-S = {data['ks']}, p = {data['p']}")
    plt.show()
