import { Text, Timeline, StyledOcticon, Link } from '@primer/react'
import {PencilIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { VarSetEvent } from '../../states/NetworkingState';

export const Container = styled.div`
    color: rgb(55, 53, 47);
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const VarEvent = (props: { event: VarSetEvent }) => {
    return (
        <Timeline.Item>
            <Timeline.Badge sx={{bg: 'done.emphasis'}}>
                <StyledOcticon icon={PencilIcon} sx={{color: 'fg.onEmphasis'}} />
            </Timeline.Badge>
            <Timeline.Body>{props.event.DataVarSetEvent.new_value}</Timeline.Body>
        </Timeline.Item>
    );
};

export { VarEvent };