resource "aws_sfn_state_machine" "claire_acquisition" {
  name       = "claire_acquisition"
  role_arn   = aws_iam_role.claire_state_machine_role.arn
  definition = data.template_file.claire_acquisition_state_machine.rendered

}

data "template_file" "claire_acquisition_state_machine" {
  template = file("state_machines/acquisition.json")

  vars = {
    detach_memory_volume      = aws_lambda_function.detach_memory_volume.arn
    is_memory_volume_detached = aws_lambda_function.is_memory_volume_detached.arn

    move_memory_volume           = aws_sfn_state_machine.claire_move_volume.arn
    create_investigation         = aws_lambda_function.create_investigation.arn
    create_evidence_extractor    = aws_lambda_function.create_evidence_extractor.arn
    poll_evidence_extractor      = aws_lambda_function.poll_evidence_extractor.arn
    prepare_memory_volume        = aws_lambda_function.prepare_memory_volume.arn
    capture_memory               = aws_lambda_function.capture_memory.arn
    upload_memory                = aws_lambda_function.upload_memory.arn
    is_command_complete          = aws_lambda_function.is_command_complete.arn
    delete_memory_volume         = aws_lambda_function.delete_memory_volume.arn
    isolate_instance             = aws_lambda_function.isolate_instance.arn
    snapshot_disks               = aws_lambda_function.snapshot_disks.arn
    terminate_evidence_extractor = aws_lambda_function.terminate_evidence_extractor.arn
  }
}

resource "aws_sfn_state_machine" "claire_move_volume" {
  name       = "claire_move_volume"
  role_arn   = aws_iam_role.claire_state_machine_role.arn
  definition = data.template_file.claire_move_volume_state_machine.rendered
}

data "template_file" "claire_move_volume_state_machine" {
  template = file("state_machines/move_memory_volume.json")

  vars = {
    detach_memory_volume      = aws_lambda_function.detach_memory_volume.arn
    is_memory_volume_detached = aws_lambda_function.is_memory_volume_detached.arn
    attach_memory_volume      = aws_lambda_function.attach_memory_volume.arn
    is_memory_volume_attached = aws_lambda_function.is_memory_volume_attached.arn
  }
}
