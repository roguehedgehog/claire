resource "aws_sfn_state_machine" "claire_state_machine" {
  name       = "claire_state_machine"
  role_arn   = aws_iam_role.claire_state_machine_role.arn
  definition = <<EOF
{
    "StartAt": "CreateInvestigation",
    "States": {
        "CreateInvestigation": {
            "Type": "Task",
            "Resource": "${aws_lambda_function.create_investigation.arn}",
            "Next": "ProceedWithInvestigation"
        },
        "ProceedWithInvestigation": {
            "Type": "Choice",
            "Choices": [
                {
                    "Variable": "$.investigation_id",
                    "StringGreaterThan": "",
                    "Next": "IsolateAndGatherEvidence"
                }
            ],
            "Default": "InvestigationAlreadyInProgress"
        },
        "IsolateAndGatherEvidence": {
            "Type": "Parallel",
            "Branches": [
                {
                    "StartAt": "CreateExtractor",
                    "States": {
                        "CreateExtractor": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.create_evidence_extractor.arn}",
                            "Next": "WaitForExtractorBoot"
                        },
                        "WaitForExtractorBoot": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollExtractor"
                        },
                        "PollExtractor": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.poll_evidence_extractor.arn}",
                            "Next": "IsExtractorReady"
                        },
                        "IsExtractorReady": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "PrepareMemoryVolume"
                                }
                            ],
                            "Default": "WaitForExtractorBoot"
                        },
                        "PrepareMemoryVolume": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.prepare_memory_volume.arn}",
                            "Next": "WaitPrepareMemoryVolume",
                            "Retry": [
                                {
                                    "ErrorEquals": [
                                        "States.ALL"
                                    ],
                                    "IntervalSeconds": 30,
                                    "MaxAttempts": 3
                                }
                            ]
                        },
                        "WaitPrepareMemoryVolume": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollPrepareMemoryVolume"
                        },
                        "PollPrepareMemoryVolume": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_command_complete.arn}",
                            "Next": "IsPrepareMemoryVolume"
                        },
                        "IsPrepareMemoryVolume": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "DetachVolumeFromExtractor"
                                }
                            ],
                            "Default": "WaitPrepareMemoryVolume"
                        },
                        "DetachVolumeFromExtractor": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.detach_memory_volume.arn}",
                            "Next": "WaitDetachVolumeFromExtractor"
                        },
                        "WaitDetachVolumeFromExtractor": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollDetachVolumeFromExtractor"
                        },
                        "PollDetachVolumeFromExtractor": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_memory_volume_detached.arn}",
                            "Next": "IsDetachVolumeFromExtractor"
                        },
                        "IsDetachVolumeFromExtractor": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "AttachVolumeToInstance"
                                }
                            ],
                            "Default": "WaitDetachVolumeFromExtractor"
                        },
                        "AttachVolumeToInstance": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.attach_memory_volume.arn}",
                            "Next": "WaitAttachVolumeToInstance"
                        },
                        "WaitAttachVolumeToInstance": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollAttachVolumeToInstance"
                        },
                        "PollAttachVolumeToInstance": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_memory_volume_attached.arn}",
                            "Next": "IsAttachVolumeToInstance"
                        },
                        "IsAttachVolumeToInstance": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "CaptureMemory"
                                }
                            ],
                            "Default": "WaitAttachVolumeToInstance"
                        },
                        "CaptureMemory": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.capture_memory.arn}",
                            "Next": "WaitCaptureMemory"
                        },
                        "WaitCaptureMemory": {
                            "Type": "Wait",
                            "Seconds": 30,
                            "Next": "PollCaptureMemory"
                        },
                        "PollCaptureMemory": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_command_complete.arn}",
                            "Next": "IsCaptureMemory"
                        },
                        "IsCaptureMemory": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "DetachFromInstance"
                                }
                            ],
                            "Default": "WaitCaptureMemory"
                        },
                        "DetachFromInstance": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.detach_memory_volume.arn}",
                            "Next": "WaitDetachFromInstance"
                        },
                        "WaitDetachFromInstance": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollIsDetachFromInstance"
                        },
                        "PollIsDetachFromInstance": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_memory_volume_detached.arn}",
                            "Next": "IsDetachFromInstance"
                        },
                        "IsDetachFromInstance": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "AttachToExtractor"
                                }
                            ],
                            "Default": "WaitDetachFromInstance"
                        },
                        "AttachToExtractor": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.attach_memory_volume.arn}",
                            "Next": "WaitAttachToExtractor"
                        },
                        "WaitAttachToExtractor": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollIsAttachToExtractor"
                        },
                        "PollIsAttachToExtractor": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_memory_volume_attached.arn}",
                            "Next": "IsAttachToExtractor"
                        },
                        "IsAttachToExtractor": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "UploadMemoryCapture"
                                }
                            ],
                            "Default": "WaitAttachToExtractor"
                        },
                        "UploadMemoryCapture": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.upload_memory.arn}",
                            "Next": "WaitUploadMemoryCapture"
                        },
                        "WaitUploadMemoryCapture": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollUploadMemoryCapture"
                        },
                        "PollUploadMemoryCapture": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_command_complete.arn}",
                            "Next": "IsUploadMemoryCapture"
                        },
                        "IsUploadMemoryCapture": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "DetachCompleteCapture"
                                }
                            ],
                            "Default": "WaitUploadMemoryCapture"
                        },
                        "DetachCompleteCapture": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.detach_memory_volume.arn}",
                            "Next": "WaitDetachCompleteCapture"
                        },
                        "WaitDetachCompleteCapture": {
                            "Type": "Wait",
                            "Seconds": 10,
                            "Next": "PollDetachCompleteCapture"
                        },
                        "PollDetachCompleteCapture": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.is_memory_volume_detached.arn}",
                            "Next": "IsDetachCompleteCapture"
                        },
                        "IsDetachCompleteCapture": {
                            "Type": "Choice",
                            "Choices": [
                                {
                                    "Variable": "$.is_ready",
                                    "BooleanEquals": true,
                                    "Next": "DeleteMemoryVolume"
                                }
                            ],
                            "Default": "WaitDetachCompleteCapture"
                        },
                        "DeleteMemoryVolume": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.delete_memory_volume.arn}",
                            "End": true
                        }
                    }
                },
                {
                    "StartAt": "IsolateInstance",
                    "States": {
                        "IsolateInstance": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.isolate_instance.arn}",
                            "End": true
                        }
                    }
                },
                {
                    "StartAt": "SnapshotDisks",
                    "States": {
                        "SnapshotDisks": {
                            "Type": "Task",
                            "Resource": "${aws_lambda_function.snapshot_disks.arn}",
                            "End": true
                        }
                    }
                }
            ],
            "Next": "TerminateExtractorInstance"
        },
        "TerminateExtractorInstance": {
            "Type": "Task",
            "Resource": "${aws_lambda_function.terminate_evidence_extractor.arn}",
            "Next": "Success"
        },
        "Success": {
            "Type": "Succeed"
        },
        "InvestigationAlreadyInProgress": {
            "Type": "Fail",
            "Cause": "This instance is already being investigated"
        }
    }
}
EOF

}
