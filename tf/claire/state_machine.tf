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
