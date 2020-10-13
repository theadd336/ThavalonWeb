# ThavalonWeb
This repo contains the front-end web application, back-end API, and database code for ThavalonWeb. For a description of Thavalon, see below. To view the rules, see the [rules page](docs/rules.md).

# What is Thavalon?
THavalon is a massive extension of the rules presented in Don Eskridge's social deception game, The Resistance: Avalon. Thavalon was designed over several years primarily by [Andrew Hitt](https://github.com/aquadrizzt/THavalon). The main point of THavalon is to provide every player of the game with a role and that to ensure every player in the game feels like they can make an impact on the result of the game. Over time, these rules have evolved in an effort to make the game both faster and more fun, as well as to fix some of the playstyles that have been established in my main testing group's metagame.

THavalon is balanced around games of 5, 7, 8, or 10 players. 6- and 9-player games are heavily imbalanced in favor of the Good team in the base game (due to the 2:1 ratio of Good to Evil roles), and THavalon's insistence on providing every player with a unique role makes these games even more difficult for Evil.

# Using This Repo
This entire repo is fully Dockerized. Certain services can be used independent of one another and have different commands to start.
* To build and launch just the front-end webapp, type `make web.`
* To build and launch just the back-end API, type `make api.`
* To build all components, type `make.`

As is, this repo is **NOT** secured for production use. It only has basic security keys meant for rapid development. Of note, the backend currently does not support TLS, and the secret used for JWT encryption is extremely weak.
