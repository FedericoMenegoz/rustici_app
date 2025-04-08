>Note: This project is shared publicly to show the source code. It won't build due to private dependencies. :(

# Rustici Project

Welcome to the Rustici Project!

## Group Members
- Federico Menegoz (WG)
- Salvatore Gilles Cassarà 
- Alberto Cimmino 

## Building and Running the App

- To build the rustici_app: `cargo run --release --bin build_project`
- Run the `rustici_app`
- Follow the prompts and enjoy!

## AI (Salvatore Gilles Cassarà)

The brain of our robot: an AI created using reinforcement learning.
In particular, it uses the q-learning algorithm which, given a set of states the robot can be in, a set of actions it can execute and a reward function that rewards the correct actions made by the robot, tries the best way to concatenate states and actions in order to reach its goal: make money.

### Q-Learning

Evrything revolves around the q-learning function:

Qn(S, A) = (1 - α) * Q(S, A) + α * (reward * γ * estimated_reward(S))

Where:
- α is called _learning rate_: how much newly learned information affect the old knowledge
- γ is called _discount factor_: how much the maximum estimated reward of the current state affects the choice of an action
- _estimated_reward(S)_: simply retarns the maximum obtainable reward given the current state

Through training, these values are optimized, improving the AI's performance.

## Visualizer 1 (Federico Menegoz)

Visualizer that either showcases a pre-trained robot completing its mission or allows you to train a robot by fine-tuning the rewards of the q-learning algorithm. Developed using Bevy Game Engine.

### Getting Started 🏁

Simply click on "Start" from the main menu! This will prompt the AI to compute the data and initiate the simulation. The default settings are:

- World size: 200
- Pre-trained robot

### Settings 🛠️

In the main menu, click on "Settings". In the settings window, you can view and change the size and the bot you would like to use.

### Training 🏋️

Click on the "Training" button and adjust the rewards as you prefer for each state. The values displayed here are the ones used to train the default robot. Once you are satisfied with the rewards you've set, simply click "Exit and Train" and wait for your bot's training to finish. A delightful tune will accompany you through the process.
If after training you want to test your robot:

1. Open the visualizer.
2. Click on "Settings".
3. Select the custom robot.
4. Return to the main menu.
5. Click "Start".
6. If the robot seems a bit slow or inefficient, don't worry! It just needs more training and attention.

### Rustici Tool & WG 🎭

Click on the "Rustici Tool & WG" to get an idea of our Planner Tool and World Generator.
Here, a simple AI will utilize our tool to explore the map and collect resources (🐟🌳🪨🪙🗑️), always choosing the most cost-effective path.
In the [Tool & WG], you can click on 'Test!':

- With no biome type selected, it will spawn the bot on a 10x10 test world to demonstrate a deterministic outcome.
- With a biome selected, it will spawn the bot on our World Generator using that particular biome type.

### Simulation Keyboard Control

- ⬆️: Music Volume Up
- ⬇️: Music Volume Down
- ➡️: Robot Moves Faster
- ⬅️: Robot Moves Slower
- 'P': Show All Map (the Robot will explore by the end of the simulation)
- 'O': Hide All Map (not yet explored)
- 'SPACE': Zoom In
- '-': Zoom Out

## Visualizer 2 (Alberto Cimmino)

This is a visualizer that showcases a pre-trained robot completing its mission. It was developed using the crate Macroquad. It's a 3D enviorement with 2D sprites.

### Simulation Keyboard Control

- ➡️: Robot Moves Faster
- ⬅️: Robot Moves Slower
- F : Toggle between Free-Camera Mode and Following-Robot Mode.
- While in Free-Camera Mode
  - WASD, Ctrl and space to move freely
  - Mouse to control the view
