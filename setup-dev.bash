#!/bin/bash

tmux new-window -c "$PWD"
tmux split-window -h "helix $PWD"
tmux resize-pane -L 95
tmux split-pane -v
tmux send-keys -t 0 'sidetree' enter; 
tmux resize-pane -D 20
tmux select-pane -t 1
