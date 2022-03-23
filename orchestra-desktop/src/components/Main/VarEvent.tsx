import { Timeline, StyledOcticon } from '@primer/react'
import { PencilIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { DataVarSetEventFormattedValue } from '../../states/NetworkingState';

export const Container = styled.div`
    color: rgba(255, 255, 255, 0.8);
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const VarEvent = (props: { event: DataVarSetEventFormattedValue }) => {
    return (
        <Timeline.Item>
            <Timeline.Badge sx={{ bg: 'done.emphasis' }}>
                <StyledOcticon icon={PencilIcon} sx={{ color: 'fg.onEmphasis' }} />
            </Timeline.Badge>
            <Timeline.Body sx={{ color: 'fg.onEmphasis' }}>Value updated: {props.event.value}</Timeline.Body>
        </Timeline.Item>
    );
};

export { VarEvent };