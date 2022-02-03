import { Text, Timeline, StyledOcticon, Link } from '@primer/react'
import {PencilIcon, DiffIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { MapDeleteEvent, MapInsertEvent, MapUpdateEvent } from '../../states/NetworkingState';

export const Container = styled.div`
    color: rgb(55, 53, 47);
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const MapEvent = (props: { event: MapUpdateEvent|MapInsertEvent|MapDeleteEvent }) => {
    let icon = DiffIcon;
    let color = 'success.emphasis';
    let label = '';
    if ('DataMapDeleteEvent' in props.event) {
        color = 'danger.emphasis';
        label = `Entry keyed with ${props.event.DataMapDeleteEvent.deleted_key} deleted`
    } else if ('DataMapUpdateEvent' in props.event) {
        color = 'accent.emphasis';
        label = `Entry keyed with ${props.event.DataMapUpdateEvent.key} updated with value ${props.event.DataMapUpdateEvent.new_value}`
    } else if ('DataMapInsertEvent' in props.event) {
        label = `Entry keyed with ${props.event.DataMapInsertEvent.inserted_key} inserted with value ${props.event.DataMapInsertEvent.inserted_value}`
    }

    return (
        <Timeline.Item>
            <Timeline.Badge sx={{bg: color}}>
                <StyledOcticon icon={icon} sx={{color: 'fg.onEmphasis'}} />
            </Timeline.Badge>
            <Timeline.Body>{label}</Timeline.Body>
        </Timeline.Item>
    );
};

export { MapEvent };