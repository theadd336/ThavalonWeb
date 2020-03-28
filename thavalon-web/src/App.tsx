import React from 'react';
import styled from '@emotion/styled'
import './static/styles.scss';
import { WebSocketManager } from './components/communication';
import { GameBoard, Header, GameInformationCollection } from "./components/gameUIComponents";
import { DocMeta } from './components/metaComponents';

const AppContainer = styled.div`
  margin: 0;
`;

declare const wsPath: string;
function App() {

  const connection = new WebSocketManager(wsPath);
  return (
    <AppContainer>
      <DocMeta />
      <Header />
      <GameBoard webSocket={connection} />
      <GameInformationCollection webSocket={connection} />
    </AppContainer>
  );
}

export default App;
