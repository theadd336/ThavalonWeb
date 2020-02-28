import React from 'react';
import styled from '@emotion/styled'
import './static/styles.scss';

import {MissionIndicatorCollection } from './components/missionIndicators';
import { Card, MissionResult } from './Core/gameConstants';

const AppContainer = styled.div`
  margin: 0;
`;

function App() {
  return (
    <AppContainer>
      <MissionIndicatorCollection 
        numMissions={5}
        missionsInfo={[
          {
            discriminator: "MissionIndicatorProps",
            missionNum: 0, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionIndicatorProps",
            missionNum: 1, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionIndicatorProps",
            missionNum: 2, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionIndicatorProps",
            missionNum: 3, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionPlaceholderProps",
            missionNum: 4, 
            numPlayersOnMisison: 5,
            requiresDoubleFail: false
          }]}>
      </MissionIndicatorCollection>
    </AppContainer>
  );
}

export default App;
