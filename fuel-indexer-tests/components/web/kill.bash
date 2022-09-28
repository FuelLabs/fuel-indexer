#!/bin/bash

kill -9 $(lsof -ti:4000)
kill -9 $(lsof -ti:8000)