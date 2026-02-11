import matplotlib.pyplot as plt
import pandas as pd

secs = 60

plt.figure(figsize=(14, 6))


sb_2000 = pd.read_csv("sparse_bullshark_2000.csv", sep=" ", header=None)
avg_sb_2000 = sb_2000.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_2000[0],
    avg_sb_2000[2] / secs,
    "o-",
    color="blue",
    label="Sparse Bullshark 2Gb/sec",
)

b_2000 = pd.read_csv("bullshark_2000.csv", sep=" ", header=None)
mean_b_2000 = b_2000[1].mean() / secs

plt.axhline(
    y=mean_b_2000,
    color="purple",
    linestyle="--",
    linewidth=2,
    label="Bullshark 2Gb/sec",
)

sb_3000 = pd.read_csv("sparse_bullshark_3000.csv", sep=" ", header=None)
avg_sb_3000 = sb_3000.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_3000[0],
    avg_sb_3000[2] / secs,
    "o-",
    color="green",
    label="Sparse Bullshark 3Gb/sec",
)

b_3000 = pd.read_csv("bullshark_3000.csv", sep=" ", header=None)
mean_b_3000 = b_3000[1].mean() / secs

plt.axhline(
    y=mean_b_3000,
    color="green",
    linestyle="--",
    linewidth=2,
    label="Bullshark 3Gb/sec",
)

sb_4000 = pd.read_csv("sparse_bullshark_4000.csv", sep=" ", header=None)
avg_sb_4000 = sb_4000.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_4000[0],
    avg_sb_4000[2] / secs,
    "o-",
    color="red",
    label="Sparse Bullshark 4Gb/sec",
)

b_4000 = pd.read_csv("bullshark_4000.csv", sep=" ", header=None)
mean_b_4000 = b_4000[1].mean() / secs

plt.axhline(
    y=mean_b_4000,
    color="red",
    linestyle="--",
    linewidth=2,
    label="Bullshark 4Gb/sec",
)

sb_5000 = pd.read_csv("sparse_bullshark_5000.csv", sep=" ", header=None)
avg_sb_5000 = sb_5000.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_5000[0],
    avg_sb_5000[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 5Gb/sec",
)

b_5000 = pd.read_csv("bullshark_5000.csv", sep=" ", header=None)
mean_b_5000 = b_5000[1].mean() / secs

plt.axhline(
    y=mean_b_5000,
    color="orange",
    linestyle="--",
    linewidth=2,
    label="Bullshark 5Gb/sec",
)

sb_6000 = pd.read_csv("sparse_bullshark_6000.csv", sep=" ", header=None)
avg_sb_6000 = sb_6000.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_6000[0],
    avg_sb_6000[2] / secs,
    "o-",
    color="brown",
    label="Sparse Bullshark 6Gb/sec",
)

b_6000 = pd.read_csv("bullshark_6000.csv", sep=" ", header=None)
mean_b_6000 = b_6000[1].mean() / secs

plt.axhline(
    y=mean_b_6000,
    color="brown",
    linestyle="--",
    linewidth=2,
    label="Bullshark 6Gb/sec",
)

# sb_7000 = pd.read_csv("sparse_bullshark_7000.csv", sep=" ", header=None)
# avg_sb_7000 = sb_7000.groupby(0)[2].mean().reset_index()

# plt.plot(
#     avg_sb_7000[0],
#     avg_sb_7000[2] / secs,
#     "o-",
#     color="cyan",
#     label="Sparse Bullshark 7Gb/sec",
# )

b_7000 = pd.read_csv("bullshark_7000.csv", sep=" ", header=None)
mean_b_7000 = b_7000[1].mean() / secs

plt.axhline(
    y=mean_b_7000,
    color="cyan",
    linestyle="--",
    linewidth=2,
    label="Bullshark 7Gb/sec",
)


plt.xticks(avg_sb_2000[0])


plt.xlabel("Sample size")
plt.ylabel("Latency (sec)")

plt.legend(loc="upper left")
plt.show()
