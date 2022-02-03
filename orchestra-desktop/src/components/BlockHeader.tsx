import {Heading, Box} from '@primer/react'

const BlockHeader = (props: { block: String }) => {

  return (
    <Box style={{ width: '100%', height: 200, backgroundColor: '#24292f' }}>
    <Box style={{ width: 300, height: 200, backgroundColor: 'rgba(0,0,0,0.5)' }}>
      <Heading sx={{ pt: 48, pl: 24, color: 'white'}}>{props.block}</Heading>
    </Box>
  </Box>
  );
};

export { BlockHeader };